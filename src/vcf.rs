//! Module for working with VCF
pub mod compress;

use std::ffi::{CString, OsString};
use std::path::{Path, PathBuf};

use rust_htslib::bcf::header::HeaderView;
use rust_htslib::bcf::{HeaderRecord, Read};
use rust_htslib::errors::Error as htslib_error;
use rust_htslib::htslib;

use crate::errors::Error;
use crate::errors::Result;

#[derive(Debug)]
pub struct VCF {
    pub reader: rust_htslib::bcf::Reader,
    tbx: *mut htslib::tbx_t,
}

impl VCF {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        match path.as_ref().to_str() {
            Some(p) if path.as_ref().exists() => Self::new(p),
            Some(p) if !path.as_ref().exists() => Err(Error::FileNotFoundError {
                path: p.to_string(),
            }),
            _ => Err(Error::FilePathError {
                path: path.as_ref().to_string_lossy().to_string(),
            }),
        }
    }

    pub fn new(path: &str) -> Result<Self> {
        if let Some(p) = VCF::tbi_path(path) {
            if !p.exists() {
                return Err(Error::IndexNotFoundError {
                    path: p.to_string_lossy().to_string(),
                });
            }
        }

        let p = CString::new(path)?;
        let tbx: *mut htslib::tbx_t = unsafe { htslib::tbx_index_load(p.as_ptr()) };

        if tbx.is_null() {
            return Err(htslib_error::Fetch.into());
        }

        Ok(VCF {
            reader: rust_htslib::bcf::Reader::from_path(path)?,
            tbx,
        })
    }

    fn tbi_path(path: &str) -> Option<PathBuf> {
        let p = Path::new(path);

        match (p.parent(), p.file_name()) {
            (Some(parent), Some(file_name)) => {
                let mut file = OsString::from(file_name);
                file.push(".tbi");
                Some(parent.join(file))
            }
            _ => None,
        }
    }

    pub fn header(&self) -> &HeaderView {
        self.reader.header()
    }

    pub fn contig(&self) -> Vec<String> {
        self.header()
            .header_records()
            .into_iter()
            .filter_map(|x| match x {
                HeaderRecord::Contig { key: _key, values } => {
                    values.get("ID").and_then(|x| Some(x.to_owned()))
                }
                _ => None,
            })
            .collect()
    }

    pub fn count(&self) -> u64 {
        let mut sum = 0;
        let mut nseq: i32 = 0;
        let seqs = unsafe { htslib::tbx_seqnames(self.tbx, &mut nseq) };

        for i in 0..nseq {
            let mut records: u64 = 0;
            let mut v: u64 = 0;

            unsafe {
                htslib::hts_idx_get_stat((*self.tbx).idx, i, &mut records, &mut v);
            }
            sum += records;
        }

        unsafe {
            libc::free(seqs as *mut libc::c_void);
        };

        sum
    }
}

impl Drop for VCF {
    fn drop(&mut self) {
        unsafe {
            htslib::tbx_destroy(self.tbx);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_path() {
        let vcf = VCF::from_path("test/dbsnp_example.vcf.gz");

        assert!(vcf.is_ok())
    }

    #[test]
    fn test_from_path_fails_for_vcf_that_does_not_exist() {
        match VCF::from_path("test/not_exist.vcf").expect_err("unexpected result") {
            Error::FileNotFoundError { path: _ } => {}
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_from_path_fails_for_vcf_without_index() {
        match VCF::from_path("test/dbsnp_example.vcf").expect_err("unexpected result") {
            Error::IndexNotFoundError { path: _ } => {}
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_contig() {
        let vcf = VCF::from_path("test/dbsnp_example.vcf.gz").expect("Error opening file.");
        let contig = vcf.contig();

        assert_eq!(contig[0], "NC_000001.10");
        assert_eq!(contig[23], "NC_000024.9");
    }

    #[test]
    fn test_count() {
        let vcf = VCF::from_path("test/dbsnp_example.vcf.gz").expect("Error opening file.");

        assert_eq!(vcf.count(), 240);
    }

    fn read_dbsnp_example_as_vec() -> Vec<rust_htslib::bcf::Record> {
        let mut vcf = VCF::from_path("test/dbsnp_example.vcf.gz").expect("Error opening file.");
        vcf.reader
            .records()
            .map(|x| x.expect("Error reading record."))
            .collect()
    }

    #[test]
    fn test_read_mono_allelic_record() {
        let records = read_dbsnp_example_as_vec();

        assert_eq!(records[0].rid(), Some(0));
        assert_eq!(records[0].pos(), 10000);
        assert_eq!(records[0].id(), b"rs1570391677".to_vec());
        assert_eq!(records[0].alleles(), vec![b"T", b"A"]);
        assert_eq!(
            records[0]
                .info(b"RS")
                .integer()
                .expect("Error reading string.")
                .expect("Missing tag")[0],
            1570391677
        );
    }

    #[test]
    fn test_read_multi_allelic_record() {
        let records = read_dbsnp_example_as_vec();

        assert_eq!(records[14].rid(), Some(1));
        assert_eq!(records[14].pos(), 10018);
        assert_eq!(records[14].id(), b"rs1558169388".to_vec());
        assert_eq!(records[14].alleles(), vec![b"C" as &[u8], b"CACA", b"CACG"]);
        assert_eq!(
            records[14]
                .info(b"RS")
                .integer()
                .expect("Error reading string.")
                .expect("Missing tag")[0],
            1558169388
        );
    }
}
