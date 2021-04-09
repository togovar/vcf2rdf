//! Module for working with bgzip and tabix
use std::ffi;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use anyhow::anyhow;
use anyhow::Result;
use bgzip::BGZFWriter;
use flate2;
use rust_htslib::htslib;

/// Compress input file to bgzip
///
/// # Arguments
///
/// * `input` - Path to input VCF.
/// * `level` - Compression level to use when compressing. From `Some(0)` (Faster) to `Some(9)` (Best). Set `None` for default level.
/// * `tabix` - Whether if to generate `.tbi` index or not.
///
/// # Example
/// ```no_run
/// use vcf2rdf::vcf::compress;
/// compress::from_path("path/to/your.vcf", None, true);
/// // => to be stored at path/to/your.vcf.gz
/// ```
pub fn from_path<P: AsRef<Path>>(input: P, level: Option<u32>, tabix: bool) -> Result<PathBuf> {
    let mut reader = BufReader::new(File::open(&input)?);

    let mut output = PathBuf::from(input.as_ref());
    output.set_extension("vcf.gz");

    from_reader(&mut reader, &output, level, tabix)?;

    Ok(output)
}

/// Compress read content to bgzip
///
/// # Arguments
///
/// * `reader` - An object that implements `BufRead`.
/// * `output` - Path to output VCF.
/// * `level` - Compression level to use when compressing. From `Some(0)` (Faster) to `Some(9)` (Best). Set `None` for default level.
/// * `tabix` - Whether if to generate `.tbi` index or not.
///
/// Example:
/// ```no_run
/// use std::io::{self, Read, BufReader};
/// use vcf2rdf::vcf::compress;
/// let mut reader = BufReader::new(io::stdin());
/// compress::from_reader(&mut reader, "path/to/your.vcf.gz", None, true);
/// // => to be stored at path/to/your.vcf.gz
/// ```
pub fn from_reader<R: BufRead, P: AsRef<Path>>(
    reader: &mut R,
    output: P,
    level: Option<u32>,
    tabix: bool,
) -> Result<()> {
    let mut buf = Vec::<u8>::new();

    let mut file = BufWriter::new(File::create(&output)?);
    let mut writer = BGZFWriter::new(&mut file, flate2::Compression::new(level.unwrap_or(6)));

    while reader.read_until(b'\n', &mut buf)? != 0 {
        writer.write_all(buf.as_ref())?;
        buf.clear();
    }

    writer.close()?;

    if tabix {
        crate::vcf::compress::tabix(&output)?;
    }

    Ok(())
}

/// Build `.tbi` index
///
/// This function just calls htslib bindings:
///
/// ```ignore
/// extern "C" {
///     pub fn tbx_index_build(
///         fn_: *const ::std::os::raw::c_char,
///         min_shift: ::std::os::raw::c_int,
///         conf: *const tbx_conf_t,
///     ) -> ::std::os::raw::c_int;
/// }
/// ```
///
/// # Arguments
///
/// * `input` - Path to input (bgzipped) VCF.
///
/// Example:
/// ```no_run
/// use vcf2rdf::vcf::compress;
/// compress::tabix("path/to/your.vcf.gz");
/// // => to be stored at path/to/your.vcf.gz.tbi
/// ```
pub fn tabix<P: AsRef<Path>>(input: P) -> Result<()> {
    match input.as_ref().to_str() {
        Some(path) => {
            let p = ffi::CString::new(path)?;
            let ret: i32 = unsafe { htslib::tbx_index_build(p.as_ptr(), 0, &htslib::tbx_conf_vcf) };

            if ret == 0 {
                Ok(())
            } else if ret == -2 {
                Err(anyhow!("[tabix] the compression of {} is not BGZF", path))
            } else {
                Err(anyhow!("tbx_index_build failed: {}", path))
            }
        }
        None => Err(anyhow!(
            "the path may contains invalid UTF-8 characters: {}",
            input.as_ref().display()
        )),
    }
}
