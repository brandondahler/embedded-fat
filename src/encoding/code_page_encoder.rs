pub trait CodePageEncoder {
    fn encode(&self, character: char) -> Option<u8>;
    fn uppercase(&self, character: char) -> char;
}
