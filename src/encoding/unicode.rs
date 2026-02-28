use core::slice::Iter;
use core::str::Chars;

#[cfg(feature = "unicode-case-folding")]
use crate::encoding::unicode::case_folding::fold_codepoint;

#[cfg(feature = "unicode-case-folding")]
mod case_folding;

pub type Utf16CodeUnit = u16;

#[derive(Clone, Debug)]
pub enum Utf16StringError {
    UnpairedSurrogateEncountered { index: usize },
}

#[derive(Clone, Debug)]
pub enum Utf16StringInputError {
    TooLong,
}

#[derive(Clone, Debug)]
pub struct Utf16String<const MAX_LENGTH: usize> {
    code_units: [Utf16CodeUnit; MAX_LENGTH],
}

impl<const MAX_LENGTH: usize> Utf16String<MAX_LENGTH> {
    pub fn new(code_units: [Utf16CodeUnit; MAX_LENGTH]) -> Result<Self, Utf16StringError> {
        let mut is_low_surrogate_required = false;

        for (index, &code_unit) in code_units.iter().enumerate() {
            if !is_low_surrogate_required {
                match code_unit {
                    0x0000 => break,
                    0xD800..=0xDBFF => is_low_surrogate_required = true,
                    0xDC00..=0xDFFF => {
                        return Err(Utf16StringError::UnpairedSurrogateEncountered { index });
                    }
                    _ => {}
                }
            } else if !matches!(code_unit, 0xDC00..=0xDFFF) {
                return Err(Utf16StringError::UnpairedSurrogateEncountered { index: index - 1 });
            }
        }

        if is_low_surrogate_required {
            return Err(Utf16StringError::UnpairedSurrogateEncountered {
                index: code_units.len() - 1,
            });
        }

        Ok(Self { code_units })
    }

    pub fn from_str(value: &str) -> Result<Self, Utf16StringInputError> {
        let mut code_units = [0; MAX_LENGTH];

        let mut code_unit_index = 0;

        for character in value.chars() {
            let codepoint = character as u32;

            // NOTE: Rust language guarantees that chars are not in the range 0xD800..=0xDFFF
            if (codepoint <= 0xFFFF) {
                ensure!(code_unit_index < MAX_LENGTH, Utf16StringInputError::TooLong);

                code_units[code_unit_index] = codepoint as u16;
                code_unit_index += 1;
            } else {
                ensure!(
                    code_unit_index + 1 < MAX_LENGTH,
                    Utf16StringInputError::TooLong
                );

                code_units[code_unit_index] = 0xD800 | (codepoint & 0x3FF) as u16;
                code_units[code_unit_index + 1] = 0xDC00 | ((codepoint >> 10) & 0x3FF) as u16;
                code_unit_index += 2;
            }
        }

        Ok(Self { code_units })
    }

    pub fn eq_ignore_case(&self, other: &Self) -> bool {
        let mut self_codepoints = Utf16Codepoints::new(&self.code_units);
        let mut other_codepoints = Utf16Codepoints::new(&other.code_units);

        loop {
            match (self_codepoints.next(), other_codepoints.next()) {
                (None, None) => return true,
                (None, Some(_)) | (Some(_), None) => return false,

                (Some(codepoint), Some(other_codepoint)) => {
                    let is_same_codepoint = codepoint == other_codepoint
                        || fold_codepoint(codepoint) == fold_codepoint(other_codepoint);

                    if !is_same_codepoint {
                        return false;
                    }
                }
            }
        }
    }
}

impl<const MAX_LENGTH: usize> PartialEq for Utf16String<MAX_LENGTH> {
    fn eq(&self, other: &Self) -> bool {
        for i in 0..MAX_LENGTH {
            let code_unit = self.code_units[i];
            if code_unit != other.code_units[i] {
                return false;
            }

            if code_unit == 0 {
                break;
            }
        }

        true
    }
}

impl<const MAX_LENGTH: usize> Eq for Utf16String<MAX_LENGTH> {}

struct Utf16Codepoints<'a> {
    code_units: Iter<'a, Utf16CodeUnit>,
}

impl<'a> Utf16Codepoints<'a> {
    fn new(code_units: &'a [Utf16CodeUnit]) -> Self {
        Self {
            code_units: code_units.iter(),
        }
    }
}

impl<'a> Iterator for Utf16Codepoints<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        let code_unit = *self.code_units.next()?;

        if code_unit == 0x0000 {
            return None;
        }

        if !matches!(code_unit, 0xD800..=0xDBFF) {
            return Some(code_unit as u32);
        }

        // SAFETY: Guaranteed to be present by Utf16String::new
        let low_surrogate_code_unit = unsafe { self.code_units.next().unwrap_unchecked() };

        let high_surrogate = code_unit & 0x3FF;
        let low_surrogate = low_surrogate_code_unit & 0x3FF;

        Some(((high_surrogate as u32) << 10) | low_surrogate as u32)
    }
}

#[cfg(not(feature = "unicode-case-folding"))]
#[inline]
fn fold_codepoint(codepoint: u32) -> u32 {
    if matches!(codepoint, 0x41..=0x5A) {
        codepoint + 32
    } else {
        codepoint
    }
}
