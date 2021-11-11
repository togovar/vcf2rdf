use std::ffi::CString;
use std::path::Path;

use rust_htslib::errors::Error as htslib_error;
use rust_htslib::htslib;
use rust_htslib::tbx;
use rust_htslib::tbx::Read;

use crate::errors::{Error, Result};
use crate::util::path;
use crate::util::vcf::tabix;

#[derive(Debug)]
pub struct Tabix {
    inner: *mut htslib::tbx_t,
    reader: tbx::Reader,
}

impl Drop for Tabix {
    fn drop(&mut self) {
        unsafe {
            htslib::tbx_destroy(self.inner);
        }
    }
}

impl Tabix {
    /// Create tabix index if not exists and open the index from `path`
    ///
    /// # Arguments
    ///
    /// * `input` - Path to input VCF (need to compressed by `bgzip`).
    ///
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Tabix> {
        if !path.as_ref().exists() {
            Err(Error::FileNotFoundError(
                path.as_ref().to_string_lossy().to_string(),
            ))?
        }

        let tbi = path::change_extension(path.as_ref(), "gz.tbi")?;

        match path.as_ref().to_str() {
            Some(p) if tbi.exists() => Self::new(p),
            Some(_) => Self::index(path.as_ref()),
            None => Err(Error::FilePathError(
                path.as_ref().to_string_lossy().to_string(),
            )),
        }
    }

    /// Create tabix index from `path`
    ///
    /// # Arguments
    ///
    /// * `input` - Path to input VCF (need to compressed by `bgzip`).
    ///
    pub fn index<P: AsRef<Path>>(path: P) -> Result<Tabix> {
        tabix::create(path.as_ref())?;

        let tbi = path::change_extension(path.as_ref(), "gz.tbi")?;

        match path.as_ref().to_str() {
            Some(p) if tbi.exists() => Self::new(p),
            Some(_) => Err(Error::FileNotFoundError(tbi.to_string_lossy().to_string())),
            _ => Err(Error::FilePathError(
                path.as_ref().to_string_lossy().to_string(),
            )),
        }
    }

    fn new(path: &str) -> Result<Tabix> {
        let inner: *mut htslib::tbx_t =
            unsafe { htslib::tbx_index_load(CString::new(path)?.as_ptr()) };

        if inner.is_null() {
            Err(htslib_error::Fetch)?;
        }

        let reader = tbx::Reader::from_path(path)?;

        Ok(Self { inner, reader })
    }

    pub fn header(&self) -> &Vec<String> {
        self.reader.header()
    }

    pub fn seqnames(&self) -> Vec<String> {
        self.reader.seqnames()
    }

    pub fn tid(&self, name: &str) -> Result<u64> {
        Ok(self.reader.tid(name)?)
    }

    pub fn count(&self) -> u64 {
        let mut sum = 0;
        let mut nseq: i32 = 0;
        let seqs: *mut *const std::os::raw::c_char =
            unsafe { htslib::tbx_seqnames(self.inner, &mut nseq) };

        for i in 0..nseq {
            let mut records: u64 = 0;
            let mut v: u64 = 0;

            unsafe {
                htslib::hts_idx_get_stat((*self.inner).idx, i, &mut records, &mut v);
            }
            sum += records;
        }

        unsafe {
            libc::free(seqs as *mut libc::c_void);
        };

        sum
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let tabix = Tabix::new("test/vcf_spec.vcf.gz").expect("Error opening tabix");

        assert_eq!(tabix.count(), 5);
    }

    #[test]
    fn test_from_path() {
        assert!(Tabix::from_path("test/vcf_spec.vcf").is_err());
        assert!(Tabix::from_path("test/vcf_spec.vcf.gz").is_ok());
    }

    #[test]
    fn test_count() {
        let tabix = Tabix::from_path("test/vcf_spec.vcf.gz").expect("Error opening tabix");

        assert_eq!(tabix.count(), 5);
    }

    #[test]
    fn test_seqnames() {
        let tabix = Tabix::from_path("test/vcf_spec.vcf.gz").expect("Error opening tabix");

        assert_eq!(tabix.seqnames(), vec![String::from("20")]);
    }
}
