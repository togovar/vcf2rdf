//! Executable for working with VCF statistics.
use structopt::StructOpt;

use vcf2rdf::cli::{Command, VCFStat};
use vcf2rdf::errors::Result;
use vcf2rdf::vcf::reader::Reader;

fn main() -> Result<()> {
    let options: VCFStat = VCFStat::from_args();

    match options.cmd {
        Command::Count { input } => {
            println!("{}", Reader::from_path(input)?.count());
        }
    }

    Ok(())
}
