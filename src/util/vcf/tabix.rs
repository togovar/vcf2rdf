use rust_htslib::htslib;

use crate::errors::{Error, Result};
use std::ffi;
use std::path::Path;

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
/// use vcf2rdf::util::vcf::tabix;
/// tabix::create("path/to/your.vcf.gz");
/// // => to be stored at path/to/your.vcf.gz.tbi
/// ```
pub fn create<P: AsRef<Path>>(input: P) -> Result<()> {
    match input.as_ref().to_str() {
        Some(path) => {
            let p = ffi::CString::new(path)?;
            let ret: i32 = unsafe { htslib::tbx_index_build(p.as_ptr(), 0, &htslib::tbx_conf_vcf) };

            if ret == 0 {
                Ok(())
            } else if ret == -2 {
                Err(Error::NotBgzipFileError(path.to_string()))?
            } else {
                Err(Error::IndexBuildFailedError(path.to_string()))?
            }
        }
        None => Err(Error::FilePathError(
            input.as_ref().to_string_lossy().to_string(),
        ))?,
    }
}
