use crate::code::{ArrayLiteral, ArrayLiteralElement};
use crate::processed_mappings::ProcessedMappings;
use crate::types::UnicodePlane;
use indenter::CodeFormatter;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt::{Display, Formatter};
use std::fmt::{UpperHex, Write};
use std::iter::chain;

pub struct FoldingImplementation {
    runs: BTreeMap<UnicodePlane, Vec<u16>>,
    entries: BTreeMap<UnicodePlane, Vec<u16>>,

    run_end_offsets: Vec<u8>,
    difference_indices: Vec<u8>,

    small_negative_differences: Vec<u8>,
    small_positive_differences: Vec<u8>,
    medium_negative_differences: Vec<u16>,
    medium_positive_differences: Vec<u16>,
}

impl FoldingImplementation {
    pub fn new(processed_mappings: ProcessedMappings) -> Self {
        let mut sorted_differences =
            Vec::from_iter(processed_mappings.differences().iter().copied());
        sorted_differences.sort();

        let differences_len = sorted_differences.len();

        let mut small_negative_differences = Vec::with_capacity(differences_len);
        let mut small_negative_difference_indices = HashMap::with_capacity(differences_len);

        let mut small_positive_differences = Vec::with_capacity(differences_len);
        let mut small_positive_difference_indices = HashMap::with_capacity(differences_len);

        let mut medium_negative_differences = Vec::with_capacity(differences_len);
        let mut medium_negative_difference_indices = HashMap::with_capacity(differences_len);

        let mut medium_positive_differences = Vec::with_capacity(differences_len);
        let mut medium_positive_difference_indices = HashMap::with_capacity(differences_len);

        for &difference in sorted_differences.iter() {
            match difference {
                -255..=-1 => {
                    small_negative_difference_indices
                        .insert(difference, small_negative_differences.len());
                    small_negative_differences.push(u8::try_from(-difference).unwrap());
                }
                0..=255 => {
                    small_positive_difference_indices
                        .insert(difference, small_positive_differences.len());
                    small_positive_differences.push(u8::try_from(difference).unwrap());
                }
                -65535..=-256 => {
                    medium_negative_difference_indices
                        .insert(difference, medium_negative_differences.len());
                    medium_negative_differences.push(u16::try_from(-difference).unwrap());
                }
                256..65535 => {
                    medium_positive_difference_indices
                        .insert(difference, medium_positive_differences.len());
                    medium_positive_differences.push(u16::try_from(difference).unwrap());
                }
                _ => panic!("Difference outside of expected range"),
            };
        }

        let mut runs = BTreeMap::new();
        let mut run_difference_indices = Vec::with_capacity(sorted_differences.len());

        let mut entries = BTreeMap::new();
        let mut entry_difference_indices = Vec::with_capacity(sorted_differences.len());

        let mut run_end_offsets = Vec::with_capacity(256);

        for run in processed_mappings.runs() {
            let unicode_plane = UnicodePlane::for_codepoint(run.starting_codepoint());
            let code_unit = run.starting_codepoint() as u16;
            let end_offset = run.end_offset();

            let difference_index = small_negative_difference_indices
                .get(&run.difference())
                .copied()
                .or_else(|| {
                    small_positive_difference_indices
                        .get(&run.difference())
                        .map(|value| value + small_negative_differences.len())
                })
                .or_else(|| {
                    medium_negative_difference_indices
                        .get(&run.difference())
                        .map(|value| {
                            value
                                + small_negative_differences.len()
                                + small_positive_differences.len()
                        })
                })
                .or_else(|| {
                    medium_positive_difference_indices
                        .get(&run.difference())
                        .map(|value| {
                            value
                                + small_negative_differences.len()
                                + small_positive_differences.len()
                                + medium_negative_differences.len()
                        })
                })
                .unwrap();

            let difference_index =
                u8::try_from(difference_index).expect("difference index overflowed");

            if end_offset == 0 {
                entries
                    .entry(unicode_plane)
                    .or_insert_with(|| Vec::with_capacity(2000))
                    .push(code_unit);

                entry_difference_indices.push(difference_index);
            } else {
                runs.entry(unicode_plane)
                    .or_insert_with(|| Vec::with_capacity(100))
                    .push(code_unit);

                run_end_offsets.push(end_offset);
                run_difference_indices.push(difference_index);
            }
        }

        Self {
            runs,
            entries,

            run_end_offsets,
            difference_indices: Vec::from_iter(chain(
                run_difference_indices.iter().copied(),
                entry_difference_indices.iter().copied(),
            )),

            small_negative_differences,
            small_positive_differences,
            medium_negative_differences,
            medium_positive_differences,
        }
    }

    fn write_differences<T: ArrayLiteralElement>(
        &self,
        f: &mut CodeFormatter<Formatter<'_>>,
        differences: &Vec<T>,
        name_prefix: &str,
        difference_type: &str,
        difference_offset: &mut usize,
    ) -> std::fmt::Result {
        let differences_length = differences.len();

        let array_literal = ArrayLiteral::new(differences, false);

        if *difference_offset > 0 {
            writeln!(
                f,
                "const {name_prefix}_DIFFERENCES_START_INDEX: u8 = {difference_offset};"
            )?;
        }

        writeln!(f, "#[rustfmt::skip]")?;
        writeln!(
            f,
            "static {name_prefix}_DIFFERENCES: [{difference_type}; {differences_length}] = {array_literal};\n"
        )?;

        *difference_offset += differences_length;

        Ok(())
    }
}

impl Display for FoldingImplementation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut f = CodeFormatter::new(f, "    ");
        let mut item_offset = 0;

        for (unicode_plane, starting_code_units) in self.runs.iter() {
            let unicode_plane_value = unicode_plane.value();
            let code_units_length = starting_code_units.len();

            let array_literal = ArrayLiteral::new(starting_code_units, true);

            writeln!(
                f,
                "const RUNS_{unicode_plane_value:02X}_ITEM_OFFSET: u8 = {item_offset};"
            )?;
            writeln!(f, "#[rustfmt::skip]")?;
            writeln!(
                f,
                "static RUNS_{unicode_plane_value:02X}: [u16; {code_units_length}] = {array_literal};\n"
            )?;

            item_offset += starting_code_units.len();
        }

        if !self.run_end_offsets.is_empty() {
            let offsets_length = self.run_end_offsets.len();

            let array_literal = ArrayLiteral::new(&self.run_end_offsets, false);

            writeln!(f, "#[rustfmt::skip]")?;
            writeln!(
                f,
                "static RUN_END_OFFSETS: [u8; {offsets_length}] = {array_literal};\n"
            )?;
        }

        for (unicode_plane, code_units) in self.entries.iter() {
            let unicode_plane_value = unicode_plane.value();
            let code_units_length = code_units.len();

            let array_literal = ArrayLiteral::new(code_units, true);

            writeln!(
                f,
                "const ENTRIES_{unicode_plane_value:02X}_ITEM_OFFSET: u8 = {item_offset};"
            )?;
            writeln!(f, "#[rustfmt::skip]")?;
            writeln!(
                f,
                "static ENTRIES_{unicode_plane_value:02X}: [u16; {code_units_length}] = {array_literal};\n"
            )?;

            item_offset += code_units.len();
        }

        if !self.difference_indices.is_empty() {
            let indices_length = self.difference_indices.len();

            let array_literal = ArrayLiteral::new(&self.difference_indices, false);

            writeln!(f, "#[rustfmt::skip]")?;
            writeln!(
                f,
                "static DIFFERENCE_INDICES: [u8; {indices_length}] = {array_literal};\n"
            )?;
        }

        let mut difference_offset = 0;

        self.write_differences(
            &mut f,
            &self.small_negative_differences,
            "SMALL_NEGATIVE",
            "u8",
            &mut difference_offset,
        )?;

        self.write_differences(
            &mut f,
            &self.small_positive_differences,
            "SMALL_POSITIVE",
            "u8",
            &mut difference_offset,
        )?;

        self.write_differences(
            &mut f,
            &self.medium_negative_differences,
            "MEDIUM_NEGATIVE",
            "u16",
            &mut difference_offset,
        )?;

        self.write_differences(
            &mut f,
            &self.medium_positive_differences,
            "MEDIUM_POSITIVE",
            "u16",
            &mut difference_offset,
        )?;

        let unicode_planes = BTreeSet::from_iter(chain(
            self.runs.keys().copied(),
            self.entries.keys().copied(),
        ));

        write!(
            f,
            "
            pub fn fold_codepoint(codepoint: u32) -> u32 {{
                // Handle ASCII range explicitly to optimize for the most common characters
                if matches!(codepoint, 0x00..=0x7F) {{
                    return match codepoint {{
                        0x0041..=0x005A => codepoint + 32,
                        _ => codepoint,
                    }};
                }}

                let unicode_plane = (codepoint >> 16) as u8;
                let code_unit = codepoint as u16;

                #[rustfmt::skip]
                let mapped_code_unit = match unicode_plane {{
            "
        )?;

        f.indent(2);
        for unicode_plane in unicode_planes {
            let unicode_plane_value = unicode_plane.value();

            write!(
                f,
                "
                0x{unicode_plane_value:02X} => {{
                    translate_code_unit(
                        code_unit,
                "
            )?;

            f.indent(2);
            if self.runs.contains_key(&unicode_plane) {
                write!(
                    f,
                    "
                    &RUNS_{unicode_plane_value:02X},
                    RUNS_{unicode_plane_value:02X}_ITEM_OFFSET as usize,
                    "
                )?;
            } else {
                write!(
                    f,
                    "
                    &[],
                    0,
                    "
                )?;
            }

            if self.entries.contains_key(&unicode_plane) {
                write!(
                    f,
                    "
                    &ENTRIES_{unicode_plane_value:02X},
                    ENTRIES_{unicode_plane_value:02X}_ITEM_OFFSET as usize
                    "
                )?;
            } else {
                write!(
                    f,
                    "
                    &[],
                    0
                    "
                )?;
            }
            f.dedent(2);

            write!(
                f,
                "
                    )
                }}
                "
            )?;
        }
        f.dedent(2);

        write!(
            f,
            "
                    _ => return codepoint,
                }};

                ((unicode_plane as u32) << 16) | mapped_code_unit as u32
            }}

            fn translate_code_unit(
                code_unit: u16,
                runs: &[u16],
                runs_item_offset: usize,
                entries: &[u16],
                entries_item_offset: usize,
            ) -> u16 {{
                let run_index = match runs.binary_search(&code_unit) {{
                    Ok(run_index) => Some(run_index),
                    Err(insertion_index) if insertion_index > 0 => {{
                        let candidate_run_index = insertion_index - 1;

                        // SAFETY: Pre-computed during code generation
                        let run_end_offset =
                            unsafe {{ *RUN_END_OFFSETS.get_unchecked(runs_item_offset + candidate_run_index) }};

                        // SAFETY: Pre-computed during code generation
                        let starting_code_unit = unsafe {{ *runs.get_unchecked(candidate_run_index) }};
                        let ending_code_unit = starting_code_unit + run_end_offset as u16;

                        if code_unit <= ending_code_unit {{
                            Some(candidate_run_index)
                        }} else {{
                            None
                        }}
                    }}
                    _ => None,
                }};

                if let Some(run_index) = run_index {{
                    return apply_difference(code_unit, runs_item_offset + run_index);
                }}

                match entries.binary_search(&code_unit) {{
                    Ok(entry_index) => apply_difference(code_unit, entries_item_offset + entry_index),
                    Err(_) => code_unit,
                }}
            }}

            fn apply_difference(code_unit: u16, item_index: usize) -> u16 {{
                // SAFETY: Pre-computed during code generation
                let difference_index = unsafe {{ *DIFFERENCE_INDICES.get_unchecked(item_index) }};

                let difference = match difference_index {{
                    0..SMALL_POSITIVE_DIFFERENCES_START_INDEX => {{
                        // SAFETY: Pre-computed during code generation
                        unsafe {{
                            -(*SMALL_NEGATIVE_DIFFERENCES.get_unchecked(difference_index as usize) as i32)
                        }}
                    }}

                    SMALL_POSITIVE_DIFFERENCES_START_INDEX..MEDIUM_NEGATIVE_DIFFERENCES_START_INDEX => {{
                        // SAFETY: Pre-computed during code generation
                        unsafe {{
                            *SMALL_POSITIVE_DIFFERENCES.get_unchecked(
                                (difference_index - SMALL_POSITIVE_DIFFERENCES_START_INDEX) as usize,
                            ) as i32
                        }}
                    }}

                    MEDIUM_NEGATIVE_DIFFERENCES_START_INDEX..MEDIUM_POSITIVE_DIFFERENCES_START_INDEX => {{
                        // SAFETY: Pre-computed during code generation
                        unsafe {{
                            -(*MEDIUM_NEGATIVE_DIFFERENCES.get_unchecked(
                                (difference_index - MEDIUM_NEGATIVE_DIFFERENCES_START_INDEX) as usize,
                            ) as i32)
                        }}
                    }}

                    _ => {{
                        // SAFETY: Pre-computed during code generation
                        unsafe {{
                            *MEDIUM_POSITIVE_DIFFERENCES.get_unchecked(
                                (difference_index - MEDIUM_POSITIVE_DIFFERENCES_START_INDEX) as usize,
                            ) as i32
                        }}
                    }}
                }};

                (code_unit as i32 + difference) as u16
            }}
            "
        )?;

        Ok(())
    }
}
