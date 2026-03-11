#[cfg(test)]
mod tests;

use crate::encoding::Utf16CodeUnit;
use core::slice::Iter;

pub struct Utf16Codepoints<'a> {
    code_units: Iter<'a, Utf16CodeUnit>,
}

impl<'a> Utf16Codepoints<'a> {
    pub fn new(code_units: &'a [Utf16CodeUnit]) -> Self {
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
