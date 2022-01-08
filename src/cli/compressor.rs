use std::path::PathBuf;

use structopt::StructOpt;

use crate::errors::Result;
use crate::util::vcf::compress;

#[derive(StructOpt, Debug)]
pub struct Options {
    /// Generate tabix index.
    #[structopt(long)]
    pub tabix: bool,

    /// Path to file to process.
    #[structopt(parse(from_os_str))]
    pub input: PathBuf,
}

pub fn run(options: Options) -> Result<()> {
    let path = compress::from_path(options.input, None, None, options.tabix)?;

    eprintln!("BGZF to {:?}", &path);

    Ok(())
}
