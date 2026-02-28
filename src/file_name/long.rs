use crate::encoding::{Utf16CodeUnit, Utf16String, Utf16StringError, Utf16StringInputError};
use core::fmt::{Display, Formatter};

pub const LONG_NAME_MAX_LENGTH: usize = 255;

#[derive(Clone, Debug)]
pub struct LongFileName {
    utf16_string: Utf16String<LONG_NAME_MAX_LENGTH>,
}

impl LongFileName {
    pub fn new(
        utf16_code_units: [Utf16CodeUnit; LONG_NAME_MAX_LENGTH],
    ) -> Result<Self, LongFileNameError> {
        let utf16_string = Utf16String::new(utf16_code_units)?;

        Ok(LongFileName { utf16_string })
    }

    pub fn from_str(name: &str) -> Result<Self, LongFileNameStringError> {
        ensure!(!name.is_empty(), LongFileNameStringError::InputEmpty);

        for (character_index, character) in name.chars().enumerate() {
            ensure!(
                Self::is_valid_character(character),
                LongFileNameStringError::CharacterInvalid {
                    character,
                    offset: character_index as u8
                }
            );
        }

        Ok(Self {
            utf16_string: Utf16String::from_str(name)?,
        })
    }

    fn is_valid_character(character: char) -> bool {
        !matches!(
            character,
            '\0'..='\x1F' | '"' | '*' | '/' | ':' | '<' | '>' | '?' | '\\' | '|' | '\u{FFFF}'
        )
    }
}

impl PartialEq for LongFileName {
    fn eq(&self, other: &Self) -> bool {
        self.utf16_string.eq_ignore_case(&other.utf16_string)
    }
}

impl Eq for LongFileName {}

#[derive(Clone, Debug)]
pub enum LongFileNameError {
    UnpairedSurrogateEncountered { offset: u8 },
}

impl Display for LongFileNameError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            LongFileNameError::UnpairedSurrogateEncountered { offset } => {
                write!(f, "unpaired surrogate encountered at offset {}", offset)
            }
        }
    }
}

impl From<Utf16StringError> for LongFileNameError {
    fn from(value: Utf16StringError) -> Self {
        match value {
            Utf16StringError::UnpairedSurrogateEncountered {
                index: code_unit_offset,
            } => LongFileNameError::UnpairedSurrogateEncountered {
                offset: code_unit_offset as u8,
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LongFileNameStringError {
    CharacterInvalid { character: char, offset: u8 },
    InputEmpty,
    InputTooLong,
}

impl From<Utf16StringInputError> for LongFileNameStringError {
    fn from(value: Utf16StringInputError) -> Self {
        match value {
            Utf16StringInputError::TooLong => LongFileNameStringError::InputTooLong,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::array::from_fn;

    mod from_str {
        use super::*;

        #[test]
        fn basic_input_parsed_correctly() {
            let mut expected_characters = [0; LONG_NAME_MAX_LENGTH];
            expected_characters[0] = 'f' as u16;
            expected_characters[1] = 'o' as u16;
            expected_characters[2] = 'o' as u16;

            let long_file_name =
                LongFileName::from_str("foo").expect("Name should parse successfully");

            assert_eq!(
                long_file_name.utf16_string,
                Utf16String::new(expected_characters).unwrap(),
                "Characters should match expected result"
            );
        }

        #[test]
        fn empty_input_returns_error() {
            let result = LongFileName::from_str("").expect_err("Err should be returned");

            assert!(
                matches!(result, LongFileNameStringError::InputEmpty),
                "Returned error should be InputEmpty"
            );
        }

        #[test]
        fn too_long_input_returns_error() {
            let result =
                LongFileName::from_str(&"a".repeat(256)).expect_err("Err should be returned");

            assert_eq!(
                result,
                LongFileNameStringError::InputTooLong,
                "Returned error should be InputTooLong"
            );
        }

        #[test]
        fn invalid_filename_character_returns_error() {
            let invalid_characters = "\
                \x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F\
                \x10\x11\x12\x13\x14\x15\x16\x17\x18\x19\x1A\x1B\x1C\x1D\x1E\x1F\
                \"*/:<>?\\|\u{FFFF}";

            for invalid_character in invalid_characters.chars() {
                let mut invalid_character_buffer = [0; 4];
                let input = invalid_character.encode_utf8(&mut invalid_character_buffer);

                let result = LongFileName::from_str(&input).expect_err("Err should be returned");

                assert!(
                    matches!(
                        result,
                        LongFileNameStringError::CharacterInvalid {
                            character: invalid_character,
                            offset: 0
                        }
                    ),
                    "Returned error should be CharacterInvalid(0x{:04X})",
                    invalid_character as u16
                );
            }
        }
    }

    mod eq {
        use super::*;

        #[test]
        fn same_case_returns_true() {
            let name_1 = LongFileName::from_str("foobar").expect("Provided string should be valid");

            assert_eq!(name_1, name_1, "Values should be equal");
        }

        #[test]
        fn max_length_returns_true() {
            let name_1 = LongFileName::from_str(&"a".repeat(LONG_NAME_MAX_LENGTH))
                .expect("Provided string should be valid");

            assert_eq!(name_1, name_1, "Values should be equal");
        }

        #[test]
        fn different_case_returns_true() {
            let name_1 = LongFileName::from_str("fooBar").expect("Provided string should be valid");
            let name_2 = LongFileName::from_str("fOobAr").expect("Provided string should be valid");

            assert_eq!(name_1, name_2, "Values should be equal");
            assert_eq!(name_2, name_1, "Values should be equal");
        }

        #[cfg(feature = "unicode-case-folding")]
        #[test]
        fn different_case_simple_folding_returns_true() {
            // Both values are folded to ß when using the simple case folding mapping
            let name_1 = LongFileName::from_str("ß").expect("Provided string should be valid");
            let name_2 = LongFileName::from_str("ẞ").expect("Provided string should be valid");

            assert_eq!(name_1, name_2, "Values should be equal");
            assert_eq!(name_2, name_1, "Values should be equal");
        }

        #[test]
        fn different_values_returns_false() {
            let name_1 = LongFileName::from_str("a").expect("Provided string should be valid");
            let name_2 = LongFileName::from_str("b").expect("Provided string should be valid");

            assert_ne!(name_1, name_2, "Values should not be equal");
            assert_ne!(name_2, name_1, "Values should not be equal");
        }

        #[test]
        fn different_lengths_returns_false() {
            let name_1 = LongFileName::from_str("foo").expect("Provided string should be valid");
            let name_2 = LongFileName::from_str("foobar").expect("Provided string should be valid");

            assert_ne!(name_1, name_2, "Values should not be equal");
            assert_ne!(name_2, name_1, "Values should not be equal");
        }

        #[test]
        fn different_complex_full_folding_returns_false() {
            // ẞ would be folded to SS when using the full case folding mapping
            let name_1 = LongFileName::from_str("ẞ").expect("Provided string should be valid");
            let name_2 = LongFileName::from_str("SS").expect("Provided string should be valid");

            assert_ne!(name_1, name_2, "Values should not be equal");
            assert_ne!(name_2, name_1, "Values should not be equal");
        }
    }
}
