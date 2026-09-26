#[cfg(feature = "unicode")]
#[path = "../../src/encoding/unicode/case_folding.rs"]
mod unicode_case_folding;

#[cfg(feature = "unicode")]
use crate::unicode_case_folding::fold_character;

fn main() {
    for i in char::MIN..=char::MAX {
        println!("{}", i);

        #[cfg(feature = "unicode")]
        println!("{}", fold_character(i));
    }
}
