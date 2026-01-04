use crate::utils::write_le_u16;
use core::char::ToUppercase;
use core::error::Error;
use core::fmt::{Display, Formatter};

#[derive(Clone, Copy, Debug)]
pub struct CharacterEncodingError(pub char);

impl Display for CharacterEncodingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "character `{}` (\\u{:08X}) does not have a valid encoding",
            self.0, self.0 as u32
        )
    }
}

impl Error for CharacterEncodingError {}

pub trait CodePageEncoder {
    fn encode(&self, character: char) -> Result<u8, CharacterEncodingError>;
    fn uppercase(&self, character: char) -> char;
}

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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Ucs2Character(u16);

impl Ucs2Character {
    pub const fn null() -> Ucs2Character {
        Ucs2Character(0)
    }

    pub const fn from_u16(value: u16) -> Option<Self> {
        // Surrogate pairs occupy a space of invalid codepoints
        if !matches!(value, 0xD800..=0xDFFF) {
            Some(Self(value))
        } else {
            None
        }
    }

    pub const fn from_char(value: char) -> Result<Self, CharacterEncodingError> {
        let codepoint = value as u32;

        if codepoint <= 0xFFFF {
            // Unwrap safe because the invalid codepoints map to invalid char values
            Ok(Self::from_u16(codepoint as u16).unwrap())
        } else {
            Err(CharacterEncodingError(value))
        }
    }

    pub const fn to_char(self) -> char {
        // Unwrap safe here because the invalid codepoints are disallowed by the constructor
        char::from_u32(self.0 as u32).unwrap()
    }

    pub const fn to_u16(self) -> u16 {
        self.0
    }
}

impl Display for Ucs2Character {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Ucs2Character> for char {
    fn from(value: Ucs2Character) -> char {
        value.to_char()
    }
}

impl From<Ucs2Character> for u16 {
    fn from(value: Ucs2Character) -> u16 {
        value.to_u16()
    }
}

impl TryFrom<char> for Ucs2Character {
    type Error = CharacterEncodingError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        Self::from_char(value)
    }
}
