#[cfg(test)]
mod tests;

use crate::CodePageEncoder;

/// An encoding which supports only the standard 7-bit ASCII characters.
#[derive(Clone, Copy, Debug, Default)]
pub struct AsciiOnlyEncoder;

impl CodePageEncoder for AsciiOnlyEncoder {
    /// Encodes ASCII characters to their equivalent 7-bit value, all others return [Option::None].
    fn encode(&self, character: char) -> Option<u8> {
        match character {
            '\0'..='\x7F' => Some(character as u8),
            _ => None,
        }
    }

    /// Converts ASCII characters to their equivalent uppercase ASCII character, all others return
    /// the original `character` value.
    fn uppercase(&self, character: char) -> char {
        character.to_ascii_uppercase()
    }
}
