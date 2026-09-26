// NOTE: File uses a non-standard name to ensure that the benchmark can load the normal implementation

use crate::encoding::unicode::case_folding::{fold_character, unoptimized_fold_character};

#[test]
fn fold_codepoint_matches_parsed_lookup() {
    for character in char::MIN..=char::MAX {
        assert_eq!(
            fold_character(character),
            unoptimized_fold_character(character),
            "Optimized result should match unoptimized result for 0x{:06X}",
            character as u32
        );
    }
}
