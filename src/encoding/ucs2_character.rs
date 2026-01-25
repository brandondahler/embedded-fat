#[cfg(feature = "unicode-case-folding")]
mod case_folding;

#[cfg(feature = "unicode-case-folding")]
use case_folding::*;

use core::fmt::{Display, Formatter};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
        if self == other {
            return true;
        }

        fold_character(self.0) == fold_character(other.0)
    }
}

impl Display for Ucs2Character {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

#[cfg(not(feature = "unicode-case-folding"))]
fn fold_character(character: u16) -> u16 {
    match character {
        0x0041..=0x005A => character + 32,
        _ => character,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;
    use core::ops::RangeInclusive;

    static VALID_RANGES: [RangeInclusive<u16>; 2] = [0x0000..=0xD7FF, 0xE000..=0xFFFF];

    mod null {
        use super::*;

        #[test]
        fn equals_zero() {
            assert_eq!(Ucs2Character::null().to_u16(), 0);
        }
    }

    mod from_u16 {
        use super::*;

        #[test]
        fn non_reserved_codepoints_valid() {
            for valid_range in &VALID_RANGES {
                for codepoint in valid_range.clone() {
                    let result =
                        Ucs2Character::from_u16(codepoint).expect("Some should be returned");

                    assert_eq!(result.to_u16(), codepoint);
                }
            }
        }

        #[test]
        fn reserved_codepoints_invalid() {
            for codepoint in 0xD800..=0xDFFF {
                let result = Ucs2Character::from_u16(codepoint);

                assert!(result.is_none(), "None should be returned");
            }
        }
    }

    mod from_char {
        use super::*;

        #[test]
        fn basic_multilingual_plane_characters_valid() {
            for valid_range in &VALID_RANGES {
                for codepoint in valid_range.clone() {
                    let character = char::from_u32(codepoint as u32).unwrap();
                    let result =
                        Ucs2Character::from_char(character).expect("Some should be returned");

                    assert_eq!(result.to_u16(), codepoint);
                    assert_eq!(result.to_char(), character);
                }
            }
        }

        #[test]
        fn non_bmp_character_invalid() {
            for codepoint in 0x01_0000..=0x10_FFFF {
                let character = char::from_u32(codepoint as u32).unwrap();
                let result = Ucs2Character::from_char(character);

                assert!(result.is_none(), "None should be returned");
            }
        }
    }

    mod eq_ignore_case {
        use super::*;

        #[test]
        fn same_case_values_are_equal() {
            for valid_range in &VALID_RANGES {
                for codepoint in valid_range.clone() {
                    let first = Ucs2Character::from_u16(codepoint).unwrap();
                    let second = Ucs2Character::from_u16(codepoint).unwrap();

                    assert!(first.eq_ignore_case(&first));
                    assert!(first.eq_ignore_case(&second));
                    assert!(second.eq_ignore_case(&first));
                }
            }
        }

        #[test]
        fn same_character_difference_case_are_equal() {
            let first = Ucs2Character::from_char('a').unwrap();
            let second = Ucs2Character::from_char('A').unwrap();

            assert!(first.eq_ignore_case(&second));
            assert!(second.eq_ignore_case(&first));
        }

        #[test]
        fn different_characters_are_not_equal() {
            let first = Ucs2Character::from_char('A').unwrap();
            let second = Ucs2Character::from_char('B').unwrap();

            assert!(!first.eq_ignore_case(&second));
            assert!(!second.eq_ignore_case(&first));
        }
    }

    mod display {
        use super::*;

        #[test]
        fn produces_non_empty_value() {
            let value = Ucs2Character::null();

            assert!(
                !value.to_string().is_empty(),
                "Display implementation should be non-empty"
            );
        }
    }

    #[cfg(not(feature = "unicode-case-folding"))]
    mod fold_character {}
}
