use crate::{CharacterEncodingError, CodePageEncoder};

#[derive(Debug, Default)]
pub struct AsciiOnlyEncoder;

impl CodePageEncoder for AsciiOnlyEncoder {
    fn encode(&self, character: char) -> Result<u8, CharacterEncodingError> {
        match character {
            '\0'..='\x7F' => Ok(character as u8),
            _ => Err(CharacterEncodingError(character)),
        }
    }

    fn uppercase(&self, character: char) -> char {
        character.to_ascii_uppercase()
    }
}
