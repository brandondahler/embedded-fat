use indenter::CodeFormatter;
use std::collections::BTreeMap;
use std::fmt::Write;
use std::fmt::{Display, Formatter};

pub struct FoldingTestSupport<'a> {
    parsed_mappings: &'a BTreeMap<u32, u32>,
}

impl<'a> FoldingTestSupport<'a> {
    pub fn new(parsed_mappings: &'a BTreeMap<u32, u32>) -> Self {
        Self { parsed_mappings }
    }
}

impl Display for FoldingTestSupport<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut f = CodeFormatter::new(f, "    ");

        let parsed_mappings_length = self.parsed_mappings.len();

        write!(
            f,
            "
            #[cfg(test)]
            static PARSED_MAPPINGS: [(u32, u32); {parsed_mappings_length}] = ["
        )?;

        f.indent(1);
        for (key, value) in self.parsed_mappings {
            write!(
                f,
                "
                (0x{key:06X}, 0x{value:06X}),
                "
            )?;
        }
        f.dedent(1);

        write!(
            f,
            "
            ];

            #[cfg(test)]
            pub fn unoptimized_fold_codepoint(codepoint: u32) -> u32 {{
                match PARSED_MAPPINGS.binary_search_by_key(&codepoint, |&(key, _)| key) {{
                    Ok(index) => PARSED_MAPPINGS[index].1,
                    Err(_) => codepoint,
                }}
            }}
            "
        )?;

        Ok(())
    }
}
