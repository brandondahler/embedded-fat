#[cfg(test)]
mod tests;

use crate::encoding::{Utf16CodeUnit, utf16_decode};
use core::slice::Iter;

pub struct Utf16Chars<I>
where
    I: Iterator<Item = Utf16CodeUnit>,
{
    code_units: I,
}

impl<I> Utf16Chars<I>
where
    I: Iterator<Item = Utf16CodeUnit>,
{
    /// Creates a new UTF-16 decoding iterator
    ///
    /// Callers are expected to perform their own validation to ensure that the underlying code
    /// units are valid.
    pub fn new(code_units: I) -> Self {
        Self { code_units }
    }
}

impl<I> Iterator for Utf16Chars<I>
where
    I: Iterator<Item = Utf16CodeUnit>,
{
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        utf16_decode(&mut self.code_units).unwrap()
    }
}
