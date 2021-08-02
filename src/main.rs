//! Executable for converting VCF to RDF.
use structopt::StructOpt;

use vcf2rdf::cli::{compressor, converter, generator, statistics, Command};
use vcf2rdf::errors::Result;

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    let command: Command = Command::from_args();

    match command {
        Command::Compress(opts) => compressor::run(opts),
        Command::Convert(opts) => converter::run(opts),
        Command::Stat(cmd) => statistics::run(cmd),
        Command::Generate(cmd) => generator::run(cmd),
    }
}
