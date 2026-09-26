#[cfg(feature = "unicode-case-folding")]
mod case_folding;

#[cfg(all(test, feature = "unicode-case-folding"))]
mod case_folding_tests;

#[cfg(feature = "unicode-case-folding")]
pub use case_folding::*;

#[cfg(not(feature = "unicode-case-folding"))]
#[inline]
pub fn fold_character(character: char) -> char {
    if matches!(character, '\x41'..='\x5A') {
        ((character as u8) + 32) as char
    } else {
        character
    }
}
