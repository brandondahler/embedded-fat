mod case_folding_file;
mod code;
mod folding_implementation;
mod folding_test_support;
mod processed_mappings;
mod types;

use crate::case_folding_file::CaseFoldingFile;
use crate::folding_implementation::FoldingImplementation;
use crate::folding_test_support::FoldingTestSupport;
use crate::processed_mappings::ProcessedMappings;
use clap::Parser;
use clio::{Input, Output};
use std::io::{BufWriter, Write};

#[derive(Clone, Debug, Parser)]
#[command(name = "unicode-casing-codegen")]
struct Args {
    #[arg(long, value_parser)]
    case_folding_file: Input,

    #[arg(long, value_parser)]
    output_file: Output,
}

fn main() {
    let mut args = Args::parse();
    let case_folding_file = CaseFoldingFile::new(args.case_folding_file);

    let parsed_mappings = case_folding_file.parse();
    let processed_mappings = ProcessedMappings::new(&parsed_mappings);

    let folding_implementation = FoldingImplementation::new(processed_mappings);
    let folding_test_support = FoldingTestSupport::new(&parsed_mappings);

    {
        let mut file = BufWriter::new(&mut args.output_file);

        write!(&mut file, "{folding_implementation}").unwrap();
        write!(&mut file, "{folding_test_support}").unwrap();
    }

    args.output_file.finish().unwrap();
}
