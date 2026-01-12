use crate::{CharacterEncodingError, CodePageEncoder};

pub struct ScriptedCodePageEncoder<F>(pub F)
where
    F: Fn(char) -> Result<u8, CharacterEncodingError>;

impl<F> CodePageEncoder for ScriptedCodePageEncoder<F>
where
    F: Fn(char) -> Result<u8, CharacterEncodingError>,
{
    fn encode(&self, character: char) -> Result<u8, CharacterEncodingError> {
        self.0(character)
    }

    fn uppercase(&self, character: char) -> char {
        character
    }
}
