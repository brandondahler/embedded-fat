use crate::directory_entry::SHORT_NAME_CHARACTER_COUNT;
use crate::{CharacterEncodingError, CodePageEncoder};
use core::error::Error;
use core::fmt::{Display, Formatter};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShortFileName {
    bytes: [u8; SHORT_NAME_CHARACTER_COUNT],
}

impl ShortFileName {
    pub fn new(bytes: [u8; SHORT_NAME_CHARACTER_COUNT]) -> Self {
        Self { bytes }
    }

    pub fn from_str<CPE>(encoder: &CPE, value: &str) -> Result<Self, ShortFileNameError>
    where
        CPE: CodePageEncoder,
    {
        ensure!(!value.is_empty(), ShortFileNameError::InputEmpty);

        let (name, extension) = match value.rsplit_once(".") {
            None => (value, ""),
            Some((name, extension)) => (name, extension),
        };

        ensure!(!name.is_empty(), ShortFileNameError::NameEmpty);

        let mut bytes = [0x20; SHORT_NAME_CHARACTER_COUNT];

        for (index, character) in name.chars().enumerate() {
            // Using index here instead of str.len() because this counts characters instead of bytes
            ensure!(index < 8, ShortFileNameError::NameTooLong);

            let mut encoded_character = Self::encode_character(encoder, character)?;
            ensure!(
                index != 0 || encoded_character != 0x20,
                ShortFileNameError::NameStartsWithSpace
            );

            if index == 0 && encoded_character == 0xE5 {
                encoded_character = 0x05;
            }

            bytes[index] = encoded_character;
        }

        for (index, character) in extension.chars().enumerate() {
            // Using index here instead of str.len() because this counts characters instead of bytes
            ensure!(index < 3, ShortFileNameError::ExtensionTooLong);

            bytes[8 + index] = Self::encode_character(encoder, character)?;
        }

        Ok(ShortFileName::new(bytes))
    }

    pub fn bytes(&self) -> &[u8; SHORT_NAME_CHARACTER_COUNT] {
        &self.bytes
    }

    pub fn checksum(&self) -> u8 {
        let mut checksum: u8 = 0;

        for character in self.bytes.iter() {
            checksum = checksum.rotate_right(1).wrapping_add(*character);
        }

        checksum
    }

    fn encode_character<CPE>(encoder: &CPE, character: char) -> Result<u8, ShortFileNameError>
    where
        CPE: CodePageEncoder,
    {
        ensure!(
            Self::is_valid_character(character),
            ShortFileNameError::CharacterNotAllowed(character)
        );

        let encoded_character = encoder.encode(encoder.uppercase(character))?;
        ensure!(
            Self::is_valid_encoded_character(encoded_character),
            ShortFileNameError::EncodedCharacterByteNotAllowed(character)
        );

        Ok(encoded_character)
    }

    fn is_valid_character(character: char) -> bool {
        !matches!(character, '\0'..='\x1F' | '"' | '*'..=',' | '.' | '/' | ':'..='?' | '['..=']' | '|')
    }

    fn is_valid_encoded_character(encoded_character: u8) -> bool {
        !matches!(encoded_character, 0x00..=0x1F | 0x22 | 0x2A..=0x2C | 0x2E | 0x2F | 0x3A..=0x3F | 0x5B..=0x5D | 0x7C)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ShortFileNameError {
    CharacterNotAllowed(char),
    EncodedCharacterByteNotAllowed(char),
    EncoderError(CharacterEncodingError),
    ExtensionTooLong,
    InputEmpty,
    NameEmpty,
    NameStartsWithSpace,
    NameTooLong,
}

impl Display for ShortFileNameError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            ShortFileNameError::CharacterNotAllowed(character) => write!(
                f,
                "the character `{character}` (\\u{:08X}) is not allowed",
                *character as u32
            ),
            ShortFileNameError::EncodedCharacterByteNotAllowed(character) => write!(
                f,
                "the resulting bytes from encoding character `{character}` (\\u{:08X}) is not allowed",
                *character as u32
            ),
            ShortFileNameError::EncoderError(e) => {
                write!(f, "an encoder error occurred: {}", e)
            }
            ShortFileNameError::ExtensionTooLong => write!(f, "extension is too long"),
            ShortFileNameError::InputEmpty => write!(f, "input string is empty"),
            ShortFileNameError::NameEmpty => write!(f, "name component is empty"),
            ShortFileNameError::NameStartsWithSpace => {
                write!(f, "name musts not start with a space")
            }
            ShortFileNameError::NameTooLong => write!(f, "name is too long"),
        }
    }
}

impl Error for ShortFileNameError {}

impl From<CharacterEncodingError> for ShortFileNameError {
    fn from(value: CharacterEncodingError) -> Self {
        Self::EncoderError(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AsciiOnlyEncoder;
    use crate::file_name::ShortFileName;
    use crate::mock::ScriptedCodePageEncoder;
    use alloc::string::String;

    mod from_str {
        use super::*;

        const INVALID_CHARACTERS: &str = "\
            \x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F\
            \x10\x11\x12\x13\x14\x15\x16\x17\x18\x19\x1A\x1B\x1C\x1D\x1E\x1F\
            \"*+,./:;<=>?[\\]|";

        #[test]
        fn values_converted_correctly() {
            #[rustfmt::skip]
            let test_values = [
                ("foo.bar",         "FOO     BAR".as_bytes()),
                ("FOO.BAR",         "FOO     BAR".as_bytes()),
                ("foo",             "FOO        ".as_bytes()),
                ("foo.",            "FOO        ".as_bytes()),
                ("PICKLE.A",        "PICKLE  A  ".as_bytes()),
                ("prettybg.big",    "PRETTYBGBIG".as_bytes()),
            ];

            for (input, expected_bytes) in test_values {
                let result = ShortFileName::from_str(&AsciiOnlyEncoder, input)
                    .expect("Parsing should succeed");

                assert_eq!(
                    result.bytes(),
                    expected_bytes,
                    "Result bytes should equal expected bytes"
                );
            }
        }

        #[test]
        fn valid_characters_allowed() {
            for byte_value in 0..=0xFF {
                if INVALID_CHARACTERS
                    .chars()
                    .any(|invalid_character| invalid_character as u8 == byte_value)
                {
                    continue;
                }

                let code_page_encoder = ScriptedCodePageEncoder(|character| {
                    if character == 'X' {
                        Ok(byte_value)
                    } else {
                        AsciiOnlyEncoder.encode(character)
                    }
                });

                let result = ShortFileName::from_str(&code_page_encoder, "AX.X")
                    .expect("Parsing should succeed");

                #[rustfmt::skip]
                assert_eq!(
                    *result.bytes(),
                    [
                        0x41, byte_value, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
                        byte_value, 0x20, 0x20
                    ],
                    "Result bytes should equal expected bytes"
                );
            }
        }

        #[test]
        fn e5_special_encoding_handled() {
            let code_page_encoder = ScriptedCodePageEncoder(|character| Ok(0xE5));

            let result = ShortFileName::from_str(&code_page_encoder, "XX.X")
                .expect("Parsing should succeed");

            #[rustfmt::skip]
            assert_eq!(
                *result.bytes(),
                [
                    0x05, 0xE5, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
                    0xE5, 0x20, 0x20
                ],
                "Result bytes should equal expected bytes"
            );
        }

        #[test]
        fn input_empty_returns_err() {
            let err =
                ShortFileName::from_str(&AsciiOnlyEncoder, "").expect_err("Parsing should fail");

            assert!(
                matches!(err, ShortFileNameError::InputEmpty),
                "Error should be InputEmpty"
            );
        }

        #[test]
        fn name_empty_returns_err() {
            let err = ShortFileName::from_str(&AsciiOnlyEncoder, ".foo")
                .expect_err("Parsing should fail");

            assert!(
                matches!(err, ShortFileNameError::NameEmpty),
                "Error should be NameEmpty"
            );
        }

        #[test]
        fn name_too_long_returns_err() {
            let err = ShortFileName::from_str(&AsciiOnlyEncoder, "123456789.abc")
                .expect_err("Parsing should fail");

            assert!(
                matches!(err, ShortFileNameError::NameTooLong),
                "Error should be NameTooLong"
            );
        }

        #[test]
        fn name_starts_with_space_returns_err() {
            let err = ShortFileName::from_str(&AsciiOnlyEncoder, " foo.txt")
                .expect_err("Parsing should fail");

            assert!(
                matches!(err, ShortFileNameError::NameStartsWithSpace),
                "Error should be NameStartsWithSpace"
            );
        }

        #[test]
        fn invalid_name_character_returns_err() {
            for character_index in 0..INVALID_CHARACTERS.len() {
                let mut character_str = String::with_capacity(5);
                character_str += &INVALID_CHARACTERS[character_index..character_index + 1];
                character_str += ".txt";

                let err = ShortFileName::from_str(&AsciiOnlyEncoder, &character_str)
                    .expect_err("Parsing should fail");

                assert!(
                    matches!(
                        err,
                        ShortFileNameError::CharacterNotAllowed(invalid_character)
                    ),
                    "Error should be CharacterNotAllowed"
                );
            }
        }

        #[test]
        fn invalid_name_encoded_byte_invalid_returns_err() {
            for character_index in 0..INVALID_CHARACTERS.len() {
                let character_byte = INVALID_CHARACTERS
                    .chars()
                    .skip(character_index)
                    .next()
                    .unwrap() as u8;

                let code_page_encoder = ScriptedCodePageEncoder(|character| {
                    if character == 'X' {
                        Ok(character_byte)
                    } else {
                        AsciiOnlyEncoder.encode(character)
                    }
                });

                let err = ShortFileName::from_str(&code_page_encoder, "X.A")
                    .expect_err("Parsing should fail");

                assert!(
                    matches!(err, ShortFileNameError::EncodedCharacterByteNotAllowed('X')),
                    "Error should be EncodedCharacterByteNotAllowed"
                );
            }
        }

        #[test]
        fn name_encoder_error_propagated() {
            let code_page_encoder = ScriptedCodePageEncoder(|character| {
                if character == 'X' {
                    Err(CharacterEncodingError('X'))
                } else {
                    AsciiOnlyEncoder.encode(character)
                }
            });

            let err = ShortFileName::from_str(&code_page_encoder, "X.A")
                .expect_err("Parsing should fail");

            assert!(
                matches!(
                    err,
                    ShortFileNameError::EncoderError(CharacterEncodingError('X'))
                ),
                "Error should be EncoderError"
            );
        }

        #[test]
        fn extension_too_long_returns_err() {
            let err = ShortFileName::from_str(&AsciiOnlyEncoder, "foo.1234")
                .expect_err("Parsing should fail");

            assert!(
                matches!(err, ShortFileNameError::ExtensionTooLong),
                "Error should be ExtensionTooLong"
            );
        }

        #[test]
        fn invalid_extension_character_returns_err() {
            for character_index in 0..INVALID_CHARACTERS.len() {
                let mut character_str = String::with_capacity(5);
                character_str += &INVALID_CHARACTERS[character_index..character_index + 1];
                character_str += ".txt";

                let err = ShortFileName::from_str(&AsciiOnlyEncoder, &character_str)
                    .expect_err("Parsing should fail");

                assert!(
                    matches!(
                        err,
                        ShortFileNameError::CharacterNotAllowed(invalid_character)
                    ),
                    "Error should be CharacterNotAllowed"
                );
            }
        }

        #[test]
        fn invalid_extension_encoded_byte_invalid_returns_err() {
            for character_index in 0..INVALID_CHARACTERS.len() {
                let character_byte = INVALID_CHARACTERS
                    .chars()
                    .skip(character_index)
                    .next()
                    .unwrap() as u8;

                let code_page_encoder = ScriptedCodePageEncoder(|character| {
                    if character == 'X' {
                        Ok(character_byte)
                    } else {
                        AsciiOnlyEncoder.encode(character)
                    }
                });

                let err = ShortFileName::from_str(&code_page_encoder, "A.X")
                    .expect_err("Parsing should fail");

                assert!(
                    matches!(err, ShortFileNameError::EncodedCharacterByteNotAllowed('X')),
                    "Error should be EncodedCharacterByteNotAllowed"
                );
            }
        }

        #[test]
        fn extension_encoder_error_propagated() {
            let code_page_encoder = ScriptedCodePageEncoder(|character| {
                if character == 'X' {
                    Err(CharacterEncodingError('X'))
                } else {
                    AsciiOnlyEncoder.encode(character)
                }
            });

            let err = ShortFileName::from_str(&code_page_encoder, "A.X")
                .expect_err("Parsing should fail");

            assert!(
                matches!(
                    err,
                    ShortFileNameError::EncoderError(CharacterEncodingError('X'))
                ),
                "Error should be EncoderError"
            );
        }
    }

    mod checksum {
        use super::*;

        #[test]
        fn matches_test_vectors() {
            #[rustfmt::skip]
            let test_vectors = [
                ("FOO.BAR",         0x53),
                ("foo",             0x88),
                ("PICKLE.A",        0x32),
                ("prettybg.big",    0x4C),
            ];

            for (input, expected_checksum) in test_vectors {
                let short_file_name = ShortFileName::from_str(&AsciiOnlyEncoder, input)
                    .expect("Parsing should succeed");

                assert_eq!(
                    short_file_name.checksum(),
                    expected_checksum,
                    "Computed checksum should match expected value"
                );
            }
        }
    }
}

#[cfg(test)]
mod error_tests {
    use super::*;
    use alloc::string::ToString;

    mod display {
        use super::*;

        #[test]
        fn produces_non_empty_value() {
            let values = [
                ShortFileNameError::CharacterNotAllowed('A'),
                ShortFileNameError::EncodedCharacterByteNotAllowed('A'),
                ShortFileNameError::EncoderError(CharacterEncodingError('A')),
                ShortFileNameError::ExtensionTooLong,
                ShortFileNameError::InputEmpty,
                ShortFileNameError::NameEmpty,
                ShortFileNameError::NameStartsWithSpace,
                ShortFileNameError::NameTooLong,
            ];

            for value in values {
                assert!(
                    !value.to_string().is_empty(),
                    "Display implementation should be non-empty"
                );
            }
        }
    }
}
