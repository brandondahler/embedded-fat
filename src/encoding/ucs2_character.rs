mod case_folding;

use case_folding::*;

use core::fmt::{Display, Formatter};

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

    pub const fn from_char(value: char) -> Option<Self> {
        let codepoint = value as u32;

        if codepoint <= 0xFFFF {
            // Unwrap safe because the invalid codepoints map to invalid char values
            Some(Self::from_u16(codepoint as u16).unwrap())
        } else {
            None
        }
    }

    pub const fn to_char(self) -> char {
        // Unwrap safe here because the invalid codepoints are disallowed by the constructor
        char::from_u32(self.0 as u32).unwrap()
    }

    pub const fn to_u16(self) -> u16 {
        self.0
    }

    pub fn eq_ignore_case(&self, other: &Ucs2Character) -> bool {
        if self.0 == other.0 {
            return true;
        }

        fold_character(self.0) == fold_character(other.0)
    }
}

impl Display for Ucs2Character {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}
