#[cfg(test)]
mod tests;

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
