use clio::Input;
use std::fmt::{Display, Formatter};
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
const MIN_RUN_SIZE: u16 = 10;

#[derive(Clone, Debug)]
pub struct CaseFolding {
    parsed_lookup: Vec<(u16, u16)>,

    optimized_lookup: Vec<(u16, u16)>,
    runs: Vec<Run>,
}

impl CaseFolding {
    pub fn parse_from(case_folding_file: &mut Input) -> CaseFolding {
        let reader = BufReader::new(case_folding_file);
        let mut parsed_lookup = Vec::with_capacity(2000);

        for line_result in reader.lines() {
            let line = line_result.as_ref().unwrap().trim_ascii();

            if line.is_empty() || line.starts_with("#") {
                continue;
            }

            let parts: Vec<&str> = line.split("; ").take(3).collect();

            if !matches!(parts[1], "C" | "S") {
                continue;
            }

            let code = u32::from_str_radix(parts[0], 16).unwrap();
            let mapping = u32::from_str_radix(parts[2], 16).unwrap();

            if code > 0xFFFF || mapping > 0xFFFF {
                continue;
            }

            parsed_lookup.push((code as u16, mapping as u16));
        }

        // Should already be sorted, but just in case
        parsed_lookup.sort_by_key(|(code, _)| *code);

        let mut optimized_lookup = Vec::with_capacity(parsed_lookup.len());
        let mut runs = Vec::with_capacity(100);

        let mut current_run: Option<Run> = None;

        for (code, mapping) in parsed_lookup.iter() {
            let code = *code;
            let mapping = *mapping;

            let difference = mapping as i32 - code as i32;

            if code <= 0x007F {
                assert!(
                    matches!(code, 0x0041..=0x005A) && difference == 32,
                    "Handling for the range [0x0000, 0x007F] is hard-coded in this code generator \
                        with the assumption that the ASCII rules will never change"
                );

                continue;
            }

            match current_run.as_mut() {
                None => {
                    current_run = Some(Run {
                        start: code,
                        end: code,
                        difference,
                    })
                }
                Some(current_run) => {
                    if code == current_run.end + 1 && difference == current_run.difference {
                        current_run.end = code;
                    } else {
                        Self::add_run(&mut optimized_lookup, &mut runs, current_run);

                        current_run.start = code;
                        current_run.end = code;
                        current_run.difference = difference;
                    }
                }
            };
        }

        if let Some(current_run) = current_run {
            Self::add_run(&mut optimized_lookup, &mut runs, &current_run);
        }

        CaseFolding {
            parsed_lookup,
            optimized_lookup,
            runs,
        }
    }

    fn add_run(optimized_lookup: &mut Vec<(u16, u16)>, runs: &mut Vec<Run>, run: &Run) {
        if run.len() > MIN_RUN_SIZE {
            runs.push(run.clone())
        } else {
            for key in run.start..=run.end {
                optimized_lookup.push((key, (key as i32 + run.difference) as u16));
            }
        }
    }
}

impl Display for CaseFolding {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "static LOOKUP: [(u16, u16); {}] = [",
            self.optimized_lookup.len()
        )?;

        for (key, value) in self.optimized_lookup.iter() {
            writeln!(f, "    (0x{key:04X}, 0x{value:04X}),")?;
        }

        writeln!(f, "];")?;
        writeln!(f)?;

        writeln!(f, "pub fn fold_character(character: u16) -> u16 {{")?;
        writeln!(
            f,
            "    // Handle ASCII range explicitly to optimize for the most common characters"
        )?;
        writeln!(f, "    if matches!(character, 0x0000..=0x007F) {{")?;
        writeln!(f, "        return match character {{")?;
        writeln!(f, "            0x0041..=0x005A => character + 32,")?;
        writeln!(f, "            _ => character,")?;
        writeln!(f, "        }};")?;
        writeln!(f, "    }}")?;
        writeln!(f)?;
        writeln!(f, "    match character {{")?;

        for run in self.runs.iter() {
            writeln!(
                f,
                "        0x{:04X}..=0x{:04X} => character {} {},",
                run.start,
                run.end,
                if run.difference > 0 { "+" } else { "-" },
                run.difference.abs()
            )?;
        }

        writeln!(f)?;
        writeln!(
            f,
            "        // Utilize binary search to find other possible matches"
        )?;
        writeln!(
            f,
            "        _ => match LOOKUP.binary_search_by_key(&character, |&(key, _)| key) {{"
        )?;
        writeln!(f, "            Ok(index) => LOOKUP[index].1,")?;
        writeln!(f, "            Err(_) => character,")?;
        writeln!(f, "        }},")?;
        writeln!(f, "    }}")?;
        writeln!(f, "}}")?;
        writeln!(f)?;

        writeln!(f, "#[cfg(test)]")?;
        writeln!(f, "pub mod tests {{")?;
        writeln!(f, "    use super::*;")?;
        writeln!(f)?;
        writeln!(
            f,
            "    static PARSED_LOOKUP: [(u16, u16); {}] = [",
            self.parsed_lookup.len()
        )?;
        for (code, mapping) in self.parsed_lookup.iter() {
            writeln!(f, "        (0x{:04X}, 0x{:04X}),", *code, *mapping)?;
        }
        writeln!(f, "    ];")?;
        writeln!(f)?;

        writeln!(f, "    #[test]")?;
        writeln!(f, "    fn fold_character_matches_parsed_lookup() {{")?;
        writeln!(f, "        for character in 0x0000..=0xFFFF {{")?;
        writeln!(f, "            assert_eq!(")?;
        writeln!(f, "                fold_character(character),")?;
        writeln!(f, "                unoptimized_fold_character(character),")?;
        writeln!(
            f,
            "                \"Optimized result should match unoptimized result for {{:04X}}\","
        )?;
        writeln!(f, "                character")?;
        writeln!(f, "            );")?;
        writeln!(f, "        }}")?;
        writeln!(f, "    }}")?;
        writeln!(f)?;
        writeln!(
            f,
            "    pub fn unoptimized_fold_character(character: u16) -> u16 {{"
        )?;
        writeln!(
            f,
            "        match PARSED_LOOKUP.binary_search_by_key(&character, |&(key, _)| key) {{"
        )?;
        writeln!(f, "            Ok(index) => PARSED_LOOKUP[index].1,")?;
        writeln!(f, "            Err(_) => character,")?;
        writeln!(f, "        }}")?;
        writeln!(f, "    }}")?;
        writeln!(f, "}}")?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
struct Run {
    start: u16,
    end: u16,
    difference: i32,
}

impl Run {
    pub fn len(&self) -> u16 {
        self.end - self.start
    }
}
