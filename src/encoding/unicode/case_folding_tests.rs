// NOTE: File uses a non-standard name to ensure that the benchmark can load the normal implementation

use crate::encoding::unicode::case_folding::{fold_codepoint, unoptimized_fold_codepoint};

#[test]
fn fold_codepoint_matches_parsed_lookup() {
    for codepoint in 0x00_0000..=0x10_FFFF {
        assert_eq!(
            fold_codepoint(codepoint),
            unoptimized_fold_codepoint(codepoint),
            "Optimized result should match unoptimized result for 0x{:06X}",
            codepoint
        );
    }
}
