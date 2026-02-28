#[cfg(feature = "unicode")]
#[path = "../../src/encoding/unicode/case_folding.rs"]
mod unicode_case_folding;

#[cfg(feature = "unicode")]
use crate::unicode_case_folding::fold_codepoint;

fn main() {
    for i in 0..=0x10_FFFFu32 {
        println!("{}", i);

        #[cfg(feature = "unicode")]
        println!("{}", fold_codepoint(i));
    }
}
