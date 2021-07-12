use crate::cli::Compress;
use crate::errors::Result;
use crate::util::vcf::compress;

pub fn run(options: Compress) -> Result<()> {
    let path = compress::from_path(options.input, None, options.tabix)?;

    eprintln!("BGZF to {:?}", &path);

    Ok(())
}
