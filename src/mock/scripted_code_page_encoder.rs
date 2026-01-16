use crate::CodePageEncoder;

pub struct ScriptedCodePageEncoder<F>(pub F)
where
    F: Fn(char) -> Option<u8>;

impl<F> CodePageEncoder for ScriptedCodePageEncoder<F>
where
    F: Fn(char) -> Option<u8>,
{
    fn encode(&self, character: char) -> Option<u8> {
        self.0(character)
    }

    fn uppercase(&self, character: char) -> char {
        character
    }
}
