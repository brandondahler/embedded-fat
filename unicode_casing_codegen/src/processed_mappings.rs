use crate::types::UnicodePlane;
use std::collections::{BTreeMap, HashSet};

pub struct ProcessedMappings {
    runs: Vec<Run>,
    differences: HashSet<i32>,
}

impl ProcessedMappings {
    pub fn new(parsed_mappings: &BTreeMap<u32, u32>) -> Self {
        let mut runs = Vec::with_capacity(2000);
        let mut differences = HashSet::with_capacity(256);

        let mut current_run: Option<Run> = None;

        for (&source_codepoint, &target_codepoint) in parsed_mappings {
            assert_eq!(
                UnicodePlane::for_codepoint(source_codepoint),
                UnicodePlane::for_codepoint(target_codepoint),
                "Applying differences in the implementation assumes that they do not result in cross-plane values"
            );

            let difference =
                i32::try_from(target_codepoint).unwrap() - i32::try_from(source_codepoint).unwrap();

            // Skip the ASCII range after validating that it matches the expected
            if matches!(source_codepoint, 0x00..=0x7F) {
                assert!(
                    matches!(source_codepoint, 0x41..=0x5A),
                    "Mappings should only exist for uppercase characters in the ASCII range"
                );
                assert_eq!(
                    difference, 32,
                    "Mappings should target the matching lowercase character"
                );
                continue;
            }

            differences.insert(difference);

            if let Some(run) = current_run.as_mut() {
                // Attempt to add the current codepoint to the existing run
                if run.try_add(source_codepoint, difference) {
                    continue;
                }

                let run = current_run.take().unwrap();
                // Add the previous run to the runs vector
                runs.push(run);
            }

            // Start a new run
            current_run = Some(Run::new(source_codepoint, difference));
        }

        if let Some(run) = current_run.take() {
            runs.push(run);
        }

        Self { runs, differences }
    }

    pub fn runs(&self) -> &Vec<Run> {
        &self.runs
    }

    pub fn differences(&self) -> &HashSet<i32> {
        &self.differences
    }
}

pub struct Run {
    starting_codepoint: u32,
    end_offset: u8,
    difference: i32,
}

impl Run {
    pub fn new(starting_codepoint: u32, difference: i32) -> Self {
        Self {
            starting_codepoint,
            end_offset: 0,
            difference,
        }
    }

    pub fn starting_codepoint(&self) -> u32 {
        self.starting_codepoint
    }

    pub fn end_offset(&self) -> u8 {
        self.end_offset
    }

    pub fn difference(&self) -> i32 {
        self.difference
    }

    fn try_add(&mut self, source_codepoint: u32, difference: i32) -> bool {
        let is_next_codepoint = source_codepoint
            == (self.starting_codepoint + self.end_offset as u32 + 1)
            && difference == self.difference;

        if !is_next_codepoint {
            return false;
        }

        if self.end_offset == u8::MAX {
            return false;
        }

        self.end_offset += 1;
        true
    }
}
