use anyhow::Result;
use structopt::StructOpt;

use vcf2rdf::cli::CLI;

fn main() -> Result<()> {
    CLI::from_args();

    Ok(())
}
