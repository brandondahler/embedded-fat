use crate::CharacterEncodingError;

pub trait CodePageEncoder {
    fn encode(&self, character: char) -> Result<u8, CharacterEncodingError>;
    fn uppercase(&self, character: char) -> char;
}
