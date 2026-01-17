mod case_folding;

use crate::case_folding::CaseFolding;
use clap::Parser;
use clio::{Input, Output};
use std::io::{BufWriter, Write};

#[derive(Clone, Debug, Parser)]
#[command(name = "ucs2-casing-codegen")]
struct Args {
    #[arg(long, value_parser)]
    case_folding_file: Input,

    #[arg(long, value_parser)]
    output_file: Output,
}

fn main() {
    let mut args = Args::parse();

    {
        let mut file = BufWriter::new(&mut args.output_file);

        write!(
            &mut file,
            "{}",
            CaseFolding::parse_from(&mut args.case_folding_file)
        )
        .unwrap();
    }

    args.output_file.finish().unwrap();
}
