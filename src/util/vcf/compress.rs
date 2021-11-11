//! Module for working with bgzip and tabix
use std::ffi::CString;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use rust_htslib::htslib;

use super::tabix;
use crate::errors::{Error, Result};

/// Compress input file to bgzip
///
/// # Arguments
///
/// * `input` - Path to input VCF.
/// * `level` - Compression level to use when compressing. From `Some(0)` (Faster) to `Some(9)` (Best). Set `None` for default level.
/// * `index` - Whether if to generate `.tbi` index or not.
///
/// # Example
/// ```no_run
/// use vcf2rdf::util::vcf::compress;
/// compress::from_path("path/to/your.vcf", None, None, true);
/// // => to be stored at path/to/your.vcf.gz
/// ```
pub fn from_path<P: AsRef<Path>>(
    input: P,
    output: Option<P>,
    level: Option<u8>,
    index: bool,
) -> Result<()> {
    let mut i = PathBuf::from(input.as_ref());
    let output = match &output {
        Some(path) => path.as_ref(),
        None => {
            i.set_extension("vcf.gz");
            i.as_path()
        }
    };

    let mut reader = BufReader::new(File::open(&input)?);

    from_reader(&mut reader, output, level, index)
}

/// Compress read content to bgzip
///
/// # Arguments
///
/// * `reader` - An object that implements `BufRead`.
/// * `output` - Path to output VCF.
/// * `level` - Compression level to use when compressing. From `Some(0)` (Faster) to `Some(9)` (Best). Set `None` for default level.
/// * `index` - Whether if to generate `.tbi` index or not.
///
/// Example:
/// ```no_run
/// use std::io::{self, Read, BufReader};
/// use vcf2rdf::util::vcf::compress;
/// let mut reader = BufReader::new(io::stdin());
/// compress::from_reader(&mut reader, "path/to/your.vcf.gz", None, true);
/// // => to be stored at path/to/your.vcf.gz
/// ```
pub fn from_reader<R: BufRead, P: AsRef<Path>>(
    reader: &mut R,
    output: P,
    level: Option<u8>,
    index: bool,
) -> Result<()> {
    let mut out_mode = Vec::new();
    out_mode.push(b'w');
    match level {
        Some(n) if n <= 9 => out_mode.push(n + b'0'),
        _ => out_mode.push(b'/'),
    };

    let path = output.as_ref().to_str().ok_or(Error::FilePathError(
        output.as_ref().to_string_lossy().to_string(),
    ))?;

    let fp: *mut htslib::BGZF = unsafe {
        htslib::bgzf_open(
            CString::new(path)?.as_ptr(),
            CString::new(out_mode)?.as_ptr(),
        )
    };

    if fp.is_null() {
        Err(Error::BgzipCreateError(
            output.as_ref().to_string_lossy().to_string(),
        ))?
    }

    while let Ok(buffer) = reader.fill_buf() {
        let length = buffer.len();
        if length == 0 {
            break;
        }

        let ret = unsafe {
            htslib::bgzf_write(
                fp,
                buffer.as_ptr() as *const std::os::raw::c_void,
                length as u64,
            )
        };

        if ret < 0 {
            Err(Error::BgzipWriteError(length))?
        }

        reader.consume(length);
    }

    if unsafe { htslib::bgzf_close(fp) } < 0 {
        Err(Error::BgzipCloseError)?
    };

    if index {
        tabix::create(&output)?;
    }

    Ok(())
}
