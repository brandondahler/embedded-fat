#[cfg(feature = "unicode-case-folding")]
mod case_folding;

#[cfg(feature = "unicode-case-folding")]
#[cfg(test)]
mod case_folding_tests;

mod utf16_codepoints;
mod utf16_string;

pub use utf16_codepoints::*;
pub use utf16_string::*;

pub type Utf16CodeUnit = u16;
