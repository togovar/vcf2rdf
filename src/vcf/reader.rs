use std::collections::HashMap;
use std::ffi::{CString, OsString};
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};

use rust_htslib::bcf;
use rust_htslib::bcf::Read;
use rust_htslib::errors::Error as htslib_error;
use rust_htslib::htslib;

use crate::errors::{Error, Result};
use crate::vcf::info::InfoKeys;

#[derive(Debug)]
pub struct Reader {
    reader: bcf::Reader,
    tbx: *mut htslib::tbx_t,
}

#[derive(Debug)]
pub struct Contig {
    pub rid: u32,
    pub name: String,
}

#[derive(Debug)]
pub enum InfoValue {
    Flag(bool),
    Integer(i32),
    Float(f32),
    String(String),
}

impl Display for InfoValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Debug)]
pub struct Info<'a> {
    pub key: &'a str,
    pub value: Vec<InfoValue>,
    pub typ: bcf::header::TagType,
    pub length: bcf::header::TagLength,
}

impl Reader {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        match path.as_ref().to_str() {
            Some(p) if path.as_ref().exists() => Self::new(p),
            Some(p) if !path.as_ref().exists() => Err(Error::FileNotFoundError(p.to_string()))?,
            _ => Err(Error::FilePathError(
                path.as_ref().to_string_lossy().to_string(),
            ))?,
        }
    }

    pub fn new(path: &str) -> Result<Self> {
        if let Some(p) = Reader::tbi_path(path) {
            if !p.exists() {
                Err(Error::IndexNotFoundError(p.to_string_lossy().to_string()))?;
            }
        }

        let p = CString::new(path)?;
        let tbx: *mut htslib::tbx_t = unsafe { htslib::tbx_index_load(p.as_ptr()) };

        if tbx.is_null() {
            Err(htslib_error::Fetch)?;
        }

        Ok(Reader {
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

    pub fn records(&mut self) -> bcf::Records<'_, bcf::Reader> {
        self.reader.records()
    }

    pub fn header(&self) -> &bcf::header::HeaderView {
        self.reader.header()
    }

    pub fn filters(&self) -> HashMap<u32, String> {
        let mut m = HashMap::new();

        self.header().header_records().iter().for_each(|hr| {
            if let bcf::HeaderRecord::Filter { values, .. } = hr {
                if let Some(v) = values.get("ID") {
                    if let Ok(id) = self.header().name_to_id(v.as_bytes()) {
                        m.insert(id.0, v.to_owned());
                    }
                }
            }
        });

        m
    }

    pub fn info_keys(&self) -> Vec<String> {
        self.header()
            .header_records()
            .iter()
            .filter_map(|hr| match hr {
                bcf::HeaderRecord::Info { values, .. } => {
                    values.get("ID").and_then(|x| Some(x.to_owned()))
                }
                _ => None,
            })
            .collect()
    }

    pub fn info_types(
        &self,
        keys: &InfoKeys,
    ) -> HashMap<String, (bcf::header::TagType, bcf::header::TagLength)> {
        let mut m = HashMap::new();

        keys.iter().for_each(|x| {
            if let Ok((typ, len)) = self.header().info_type(x.as_bytes()) {
                m.insert(x.to_owned(), (typ, len));
            }
        });

        m
    }

    pub fn contig(&self) -> Vec<Contig> {
        self.header()
            .header_records()
            .into_iter()
            .filter_map(|x| match x {
                bcf::HeaderRecord::Contig { key: _key, values } => values.get("ID").and_then(|x| {
                    self.header()
                        .name2rid(x.as_bytes())
                        .and_then(|i| {
                            Ok(Contig {
                                rid: i,
                                name: x.to_owned(),
                            })
                        })
                        .ok()
                }),
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

impl Drop for Reader {
    fn drop(&mut self) {
        unsafe {
            htslib::tbx_destroy(self.tbx);
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::anyhow;

    use super::*;

    #[test]
    fn test_from_path() {
        let vcf = Reader::from_path("test/dbsnp_example.vcf.gz");

        assert!(vcf.is_ok())
    }

    #[test]
    fn test_from_path_fails_for_vcf_that_does_not_exist() {
        let p = "test/not_exist.vcf";

        let expect = anyhow!(Error::FileNotFoundError(String::from(p)));
        let err = Reader::from_path(p).expect_err("unexpected result");

        assert_eq!(expect.to_string(), err.to_string());
    }

    #[test]
    fn test_from_path_fails_for_vcf_without_index() {
        let p = "test/dbsnp_example.vcf";

        let expect = anyhow!(Error::IndexNotFoundError(format!("{}.tbi", p)));
        let err = Reader::from_path(p).expect_err("unexpected result");

        assert_eq!(expect.to_string(), err.to_string());
    }

    #[test]
    fn test_info_keys() {
        let vcf = Reader::from_path("test/dbsnp_example.vcf.gz").expect("Error opening file.");
        let keys = vcf.info_keys();

        assert_eq!(keys[0], "RS".to_string());
        assert_eq!(keys[30], "CLNACC".to_string());
        assert_eq!(keys.len(), 31);
    }

    #[test]
    fn test_info_types() {
        let vcf = Reader::from_path("test/dbsnp_example.vcf.gz").expect("Error opening file.");
        let keys = vec!["RS".to_string(), "NOT_FOUND".to_string()];
        let info_types = vcf.info_types(&InfoKeys::from(&keys));

        assert_eq!(
            info_types.get("RS").expect("Error obtaining info type"),
            &(
                bcf::header::TagType::Integer,
                bcf::header::TagLength::Fixed(1)
            )
        );
        assert!(info_types.get("NOT_FOUND").is_none());
    }

    #[test]
    fn test_contig() {
        let vcf = Reader::from_path("test/dbsnp_example.vcf.gz").expect("Error opening file.");
        let contig = vcf.contig();

        assert_eq!(contig[0].name, "NC_000001.10");
        assert_eq!(contig[23].name, "NC_000024.9");
    }

    #[test]
    fn test_count() {
        let vcf = Reader::from_path("test/dbsnp_example.vcf.gz").expect("Error opening file.");

        assert_eq!(vcf.count(), 250);
    }

    fn read_vcf_as_vec<P: AsRef<Path>>(path: P) -> Vec<rust_htslib::bcf::Record> {
        let mut vcf = Reader::from_path(path).expect("Error opening file.");
        vcf.reader
            .records()
            .map(|x| x.expect("Error reading record."))
            .collect()
    }

    fn read_dbsnp_example_as_vec() -> Vec<rust_htslib::bcf::Record> {
        read_vcf_as_vec("test/dbsnp_example.vcf.gz")
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

    fn read_dbsnp_example_2_as_vec() -> Vec<rust_htslib::bcf::Record> {
        read_vcf_as_vec("test/dbsnp_example_wo_info_metadata.vcf.gz")
    }

    #[test]
    fn test_read_undefined_info() {
        let records = read_dbsnp_example_2_as_vec();

        let vec = records[0]
            .info(b"RS")
            .string()
            .expect("Error reading string.")
            .expect("Missing tag")[0];

        assert_eq!("1570391677", unsafe {
            String::from_utf8_unchecked(vec.to_vec())
        });
    }
}
