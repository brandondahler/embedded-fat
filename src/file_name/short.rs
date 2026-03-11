mod error;
mod parse_error;

#[cfg(test)]
mod tests;

pub use error::*;
pub use parse_error::*;

use crate::CodePageEncoder;
use crate::directory_entry::SHORT_NAME_CHARACTER_COUNT;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShortFileName {
    bytes: [u8; SHORT_NAME_CHARACTER_COUNT],
}

impl ShortFileName {
    pub fn new(bytes: [u8; SHORT_NAME_CHARACTER_COUNT]) -> Result<Self, ShortFileNameError> {
        for (index, character) in bytes.iter().enumerate() {
            let is_valid_character = match character {
                0x00..=0x1F | 0x22 | 0x2A..=0x2C | 0x2F | 0x3A..=0x3F | 0x5B..=0x5D | 0x7C => false,
                0x20 => index != 0,
                _ => true,
            };

            ensure!(
                is_valid_character,
                ShortFileNameError::CharacterInvalid {
                    character: *character,
                    offset: index as u8
                }
            );
        }

        Ok(Self { bytes })
    }

    pub fn from_str<CPE>(encoder: &CPE, value: &str) -> Result<Self, ShortFileNameParseError>
    where
        CPE: CodePageEncoder,
    {
        ensure!(!value.is_empty(), ShortFileNameParseError::InputEmpty);

        let (name, extension) = match value.split_once(".") {
            None => (value, ""),
            Some((name, extension)) => (name, extension),
        };

        ensure!(!name.is_empty(), ShortFileNameParseError::NameEmpty);

        let mut bytes = [0x20; SHORT_NAME_CHARACTER_COUNT];

        let mut name_len = 0;
        for (index, character) in name.chars().enumerate() {
            // Using index here instead of str.len() because this counts characters instead of bytes
            ensure!(index < 8, ShortFileNameParseError::NameTooLong);

            let mut encoded_character = Self::encode_character(encoder, character, index as u8)?;
            ensure!(
                index != 0 || encoded_character != 0x20,
                ShortFileNameParseError::NameStartsWithSpace
            );

            bytes[index] = encoded_character;
            name_len += 1;
        }

        for (index, character) in extension.chars().enumerate() {
            // Using index here instead of str.len() because this counts characters instead of bytes
            ensure!(index < 3, ShortFileNameParseError::ExtensionTooLong);

            bytes[8 + index] =
                Self::encode_character(encoder, character, name_len + 1 + index as u8)?;
        }

        Ok(Self { bytes })
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

    fn encode_character<CPE>(
        encoder: &CPE,
        character: char,
        offset: u8,
    ) -> Result<u8, ShortFileNameParseError>
    where
        CPE: CodePageEncoder,
    {
        ensure!(
            Self::is_valid_character(character),
            ShortFileNameParseError::CharacterNotAllowed { character, offset }
        );

        let encoded_character = encoder
            .encode(encoder.uppercase(character))
            .ok_or(ShortFileNameParseError::CharacterNotEncodable { character, offset })?;

        ensure!(
            Self::is_valid_encoded_character(encoded_character),
            ShortFileNameParseError::EncodedCharacterByteNotAllowed {
                character,
                encoded_character,
                offset
            }
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
