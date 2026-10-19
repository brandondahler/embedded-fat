/// The 8-bit encoding used to encode the short names within directory entries.
pub trait CodePageEncoder {
    /// Converts the provided character to its 8-bit encoded value, or [Option::None] if there is
    /// no matching encoding.
    fn encode(&self, character: char) -> Option<u8>;

    /// Converts the provided character to its equivalent uppercase character.  If no equivalent
    /// uppercase character exists within the encoding or the character is not handled by this
    /// encoding, the provided `character` should be returned.
    ///
    /// Encodings are not required to follow Unicode or other applicable standards when determining
    /// what characters are uppercased to other characters.
    fn uppercase(&self, character: char) -> char;
}
