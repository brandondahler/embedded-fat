use clio::Input;
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader};

// * Each lookup table entry takes up 4 bytes
// * Explicit range handling takes on the order of 20 bytes (architecture dependent) and will add a
//   new branch that must be individually be checked outside the lookup table.
// * Consequently, using a minimum size less than 5 results in more bytes in extra CPU instructions
//   than space saved.
//
// For CaseFolding 17.0.0
// * There are 1,179 table entries without explicit range handling
//   * A binary search of this table requires 11 comparisons to locate the correct entry if it
//     exists.
// * There are 704 table entries with explicit range handling (MIN_RUN_SIZE = 5)
//   * A binary search of this table requires 10 comparisons
//   * Unfortunately, the explicit range handling adds 28 additional comparisons ahead of the
//     binary search.
// * Based on these two extremes, there should be a MIN_RUN_SIZE value which provides a nice balance
//   between reducing the lookup table size while minimizing the additional comparisons for the
//   explicit range handling.
// * More than half of the explicit ranges have a length less than 10
//   * Using this limit results in 776 entries, binary search requiring 10 comparisons and explicit
//     range handling requiring only 12 extra comparisons.

#[derive(Clone, Debug)]
pub struct CaseFoldingFile {
    input: Input,
}

impl CaseFoldingFile {
    pub fn new(input: Input) -> Self {
        Self { input }
    }

    /// Reads the provided file and returns the ordered set of parsed entries
    pub fn parse(self) -> BTreeMap<u32, u32> {
        let reader = BufReader::new(self.input);
        let mut parsed_mappings = BTreeMap::new();

        for line_result in reader.lines() {
            let line = line_result.as_ref().unwrap().trim_ascii();

            if line.is_empty() || line.starts_with("#") {
                continue;
            }

            let parts: Vec<&str> = line.split("; ").take(3).collect();

            if !matches!(parts[1], "C" | "S") {
                continue;
            }

            let source_codepoint = u32::from_str_radix(parts[0], 16).unwrap();
            let target_codepoint = u32::from_str_radix(parts[2], 16).unwrap();

            parsed_mappings.insert(source_codepoint, target_codepoint);
        }

        parsed_mappings
    }
}
