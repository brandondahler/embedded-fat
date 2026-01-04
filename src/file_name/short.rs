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

    pub fn write(&self, bytes: &mut [u8]) {
        bytes[0..11].copy_from_slice(&self.bytes);
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
        !matches!(character, '\0'..='\x1F' | '"' | '*'..=',' | '.'..='/' | ':'..='?' | '['..=']' | '|')
    }

    fn is_valid_encoded_character(encoded_character: u8) -> bool {
        !matches!(encoded_character, 0x00..=0x1F | 0x22 | 0x2A..=0x2C | 0x2F | 0x3A..=0x3F | 0x5B..=0x5D | 0x7C)
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
