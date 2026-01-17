use core::error::Error;
use core::fmt::{Display, Formatter};

#[derive(Clone, Debug)]
pub enum ShortFileNameParseError {
    CharacterNotAllowed {
        character: char,
        offset: u8,
    },
    CharacterNotEncodable {
        character: char,
        offset: u8,
    },
    EncodedCharacterByteNotAllowed {
        character: char,
        encoded_character: u8,
        offset: u8,
    },
    ExtensionTooLong,
    InputEmpty,
    NameEmpty,
    NameStartsWithSpace,
    NameTooLong,
}

impl Display for ShortFileNameParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            ShortFileNameParseError::CharacterNotAllowed { character, offset } => write!(
                f,
                "the character `{character}` (\\u{:08X}) at offset {offset} is not allowed",
                *character as u32
            ),
            ShortFileNameParseError::CharacterNotEncodable { character, offset } => write!(
                f,
                "the character `{character}` (\\u{:08X}) at offset {offset} is not ecodable by the configured encoder",
                *character as u32
            ),
            ShortFileNameParseError::EncodedCharacterByteNotAllowed {
                character,
                encoded_character,
                offset,
            } => {
                write!(
                    f,
                    "the resulting encoded byte 0x{encoded_character:02X} for character `{character}` (\\u{:08X}) at offset {offset} is not allowed",
                    *character as u32
                )
            }
            ShortFileNameParseError::ExtensionTooLong => write!(f, "extension is too long"),
            ShortFileNameParseError::InputEmpty => write!(f, "input string is empty"),
            ShortFileNameParseError::NameEmpty => write!(f, "name component is empty"),
            ShortFileNameParseError::NameStartsWithSpace => {
                write!(f, "name musts not start with a space")
            }
            ShortFileNameParseError::NameTooLong => write!(f, "name is too long"),
        }
    }
}

impl Error for ShortFileNameParseError {}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    mod display {
        use super::*;

        #[test]
        fn produces_non_empty_value() {
            let values = [
                ShortFileNameParseError::CharacterNotAllowed {
                    character: 'A',
                    offset: 0,
                },
                ShortFileNameParseError::CharacterNotEncodable {
                    character: 'A',
                    offset: 0,
                },
                ShortFileNameParseError::EncodedCharacterByteNotAllowed {
                    character: 'A',
                    encoded_character: b'A',
                    offset: 0,
                },
                ShortFileNameParseError::ExtensionTooLong,
                ShortFileNameParseError::InputEmpty,
                ShortFileNameParseError::NameEmpty,
                ShortFileNameParseError::NameStartsWithSpace,
                ShortFileNameParseError::NameTooLong,
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
