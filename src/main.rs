use anyhow::Result;
use structopt::StructOpt;

use vcf2rdf::cli::Options;

fn main() -> Result<()> {
    Options::from_args();

    Ok(())
}
