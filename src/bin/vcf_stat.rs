use anyhow::Result;
use structopt::StructOpt;

use vcf2rdf::cli::{Command, VCFStat};
use vcf2rdf::vcf::VCF;

fn main() -> Result<()> {
    let options: VCFStat = VCFStat::from_args();

    match options.cmd {
        Command::Count { input } => {
            println!("{}", VCF::from_path(input)?.count());
        }
    }

    Ok(())
}
