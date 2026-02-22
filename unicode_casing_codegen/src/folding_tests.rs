use indenter::CodeFormatter;
use std::collections::BTreeMap;
use std::fmt::Write;
use std::fmt::{Display, Formatter};

pub struct FoldingTests<'a> {
    parsed_mappings: &'a BTreeMap<u32, u32>,
}

impl<'a> FoldingTests<'a> {
    pub fn new(parsed_mappings: &'a BTreeMap<u32, u32>) -> Self {
        Self { parsed_mappings }
    }
}

impl Display for FoldingTests<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut f = CodeFormatter::new(f, "    ");

        let parsed_mappings_length = self.parsed_mappings.len();

        write!(
            f,
            "
            #[cfg(test)]
            pub mod tests {{
                use super::*;

                static PARSED_MAPPINGS: [(u32, u32); {parsed_mappings_length}] = ["
        )?;

        f.indent(2);
        for (key, value) in self.parsed_mappings {
            write!(
                f,
                "
                (0x{key:06X}, 0x{value:06X}),
                "
            )?;
        }
        f.dedent(2);

        write!(
            f,
            "
                ];

                #[test]
                fn fold_codepoint_matches_parsed_lookup() {{
                    for codepoint in 0x00_0000..=0x10_FFFF {{
                        assert_eq!(
                            fold_codepoint(codepoint),
                            unoptimized_fold_codepoint(codepoint),
                            \"Optimized result should match unoptimized result for 0x{{:06X}}\",
                            codepoint
                        );
                    }}
                }}

                pub fn unoptimized_fold_codepoint(codepoint: u32) -> u32 {{
                    match PARSED_MAPPINGS.binary_search_by_key(&codepoint, |&(key, _)| key) {{
                        Ok(index) => PARSED_MAPPINGS[index].1,
                        Err(_) => codepoint,
                    }}
                }}
            }}
            "
        )?;

        Ok(())
    }
}
