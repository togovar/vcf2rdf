use std::path::PathBuf;

use structopt::StructOpt;

use crate::errors::Result;
use crate::vcf::reader::Reader;

#[derive(StructOpt, Debug)]
pub enum Options {
    /// Counts records.
    Count {
        /// Path to file to process.
        #[structopt(name = "FILE", parse(from_os_str))]
        input: PathBuf,
    },
}

pub fn run(command: Options) -> Result<()> {
    match command {
        Options::Count { input } => println!("{}", Reader::from_path(input)?.count()),
    }

    Ok(())
}
