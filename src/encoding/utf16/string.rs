mod error;
mod input_error;

#[cfg(test)]
mod tests;

pub use error::*;
pub use input_error::*;

use crate::encoding::{Utf16Chars, Utf16CodeUnit, fold_character, utf16_encode};

#[derive(Clone, Debug)]
pub struct Utf16String<const MAX_LENGTH: usize> {
    code_units: [Utf16CodeUnit; MAX_LENGTH],
}

impl<const MAX_LENGTH: usize> Utf16String<MAX_LENGTH> {
    pub fn new(code_units: [Utf16CodeUnit; MAX_LENGTH]) -> Result<Self, Utf16StringError> {
        let mut is_low_surrogate_required = false;

        for (index, &code_unit) in code_units.iter().enumerate() {
            if !is_low_surrogate_required {
                match code_unit {
                    0x0000 => break,
                    0xD800..=0xDBFF => is_low_surrogate_required = true,
                    0xDC00..=0xDFFF => {
                        return Err(Utf16StringError::UnpairedSurrogateEncountered { index });
                    }
                    _ => {}
                }
            } else if matches!(code_unit, 0xDC00..=0xDFFF) {
                is_low_surrogate_required = false;
            } else {
                return Err(Utf16StringError::UnpairedSurrogateEncountered { index: index - 1 });
            }
        }

        if is_low_surrogate_required {
            // NOTE: `code_units.len() - 1` will be correct despite the break above for 0x0000
            //   because the break only occurs when `is_low_surrogate_required == false`.
            return Err(Utf16StringError::UnpairedSurrogateEncountered {
                index: code_units.len() - 1,
            });
        }

        Ok(Self { code_units })
    }

    pub fn from_str(value: &str) -> Result<Self, Utf16StringInputError> {
        let mut code_units = [0; MAX_LENGTH];

        let mut code_unit_index = 0;

        for (character_index, character) in value.chars().enumerate() {
            code_unit_index += utf16_encode(&mut code_units[code_unit_index..], character)?;
        }

        Ok(Self { code_units })
    }

    pub fn eq_ignore_case(&self, other: &Self) -> bool {
        let mut self_characters = self.chars();
        let mut other_characters = other.chars();

        // NOTE: candidate for self_characters.eq_by(other_characters, ...) replacement once stabilized
        loop {
            match (self_characters.next(), other_characters.next()) {
                (Some(self_character), Some(other_character)) => {
                    let is_same_character = self_character == other_character
                        || fold_character(self_character) == fold_character(other_character);

                    if !is_same_character {
                        return false;
                    }
                }
                (None, None) => return true,

                // NOTE: This isn't logically be possible; however, in theory, it could be possible
                //   if case folding caused a BMP character to be converted to a non-BMP character
                //   or vice versa.  While no such case folding entry exists as of writing, there's
                //   nothing stopping the Unicode standard from adding an entry that does so later.
                (None, Some(_)) | (Some(_), None) => return false,
            }
        }
    }

    fn chars(&self) -> impl Iterator<Item = char> {
        Utf16Chars::new(self.code_units.iter().copied())
    }
}

impl<const MAX_LENGTH: usize> PartialEq for Utf16String<MAX_LENGTH> {
    fn eq(&self, other: &Self) -> bool {
        self.chars().eq(other.chars())
    }
}

impl<const MAX_LENGTH: usize> Eq for Utf16String<MAX_LENGTH> {}
