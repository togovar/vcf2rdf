//! Executable for converting VCF to RDF.
use structopt::StructOpt;

use vcf2rdf::cli::{converter, generator, statistics, Command};
use vcf2rdf::errors::Result;

fn main() -> Result<()> {
    let command: Command = Command::from_args();

    match command {
        Command::Convert(opts) => converter::run(opts),
        Command::Stat(cmd) => statistics::run(cmd),
        Command::Generate(cmd) => generator::run(cmd),
    }
}
