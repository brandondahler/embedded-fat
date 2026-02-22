#[cfg(feature = "ucs2")]
#[path = "../../src/encoding/ucs2_character/case_folding.rs"]
mod ucs2_character_case_folding;

#[cfg(feature = "unicode")]
#[path = "../../src/encoding/unicode/case_folding.rs"]
mod unicode_case_folding;

#[cfg(feature = "ucs2")]
use crate::ucs2_character_case_folding::fold_character;

#[cfg(feature = "unicode")]
use crate::unicode_case_folding::fold_codepoint;

fn main() {
    for i in 0..=0x10_FFFFu32 {
        println!("{}", i);

        #[cfg(feature = "ucs2")]
        println!("{}", fold_character(i as u16));

        #[cfg(feature = "unicode")]
        println!("{}", fold_codepoint(i));
    }
}
