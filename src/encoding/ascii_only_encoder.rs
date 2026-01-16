use crate::CodePageEncoder;

#[derive(Debug, Default)]
pub struct AsciiOnlyEncoder;

impl CodePageEncoder for AsciiOnlyEncoder {
    fn encode(&self, character: char) -> Option<u8> {
        match character {
            '\0'..='\x7F' => Some(character as u8),
            _ => None,
        }
    }

    fn uppercase(&self, character: char) -> char {
        character.to_ascii_uppercase()
    }
}
