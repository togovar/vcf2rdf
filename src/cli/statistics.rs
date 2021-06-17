use crate::cli::Statistics;
use crate::errors::Result;
use crate::vcf::reader::Reader;

pub fn run(command: Statistics) -> Result<()> {
    match command {
        Statistics::Count { input } => println!("{}", Reader::from_path(input)?.count()),
    }

    Ok(())
}
