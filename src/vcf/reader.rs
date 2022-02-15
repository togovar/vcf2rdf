use std::collections::BTreeMap;
use std::ffi::{CString, OsString};
use std::fs::File;
use std::io;
use std::io::{BufRead, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use log::*;
use rust_htslib::bcf;
use rust_htslib::bcf::Read;
use rust_htslib::errors::Error as htslib_error;
use rust_htslib::htslib;
use tempfile::TempDir;

use crate::config::Sequence;
use crate::errors::{Error, Result};
use crate::vcf::record;

const FILENAME_STDIN: &'static str = "stdin.vcf";

#[derive(Debug)]
pub struct ReaderBuilder {
    info_keys: Option<Vec<String>>,
    references: BTreeMap<String, Option<Sequence>>,
    normalize: bool,
}

impl ReaderBuilder {
    pub fn new() -> Self {
        ReaderBuilder {
            info_keys: None,
            references: Default::default(),
            normalize: true,
        }
    }

    pub fn info_keys(mut self, keys: Vec<String>) -> Self {
        self.info_keys = Some(keys);
        self
    }

    pub fn reference(mut self, reference: BTreeMap<String, Option<Sequence>>) -> Self {
        self.references = reference;
        self
    }

    pub fn normalize(mut self, flag: bool) -> Self {
        self.normalize = flag;
        self
    }

    pub fn path<P: AsRef<Path>>(&self, path: P) -> Result<Reader> {
        match path.as_ref().to_str() {
            Some(p) if path.as_ref().exists() => self.build(p, None),
            Some(p) if !path.as_ref().exists() => Err(Error::FileNotFoundError(p.to_string()))?,
            _ => Err(Error::FilePathError(
                path.as_ref().to_string_lossy().to_string(),
            ))?,
        }
    }

    pub fn stdin(&self) -> Result<Reader> {
        let stdin = io::stdin();
        let stdin = stdin.lock();

        self.reader(stdin)
    }

    pub fn reader<R: BufRead>(&self, mut reader: R) -> Result<Reader> {
        let tmp_dir = TempDir::new()?;
        debug!("Create temporary directory: {:?}", tmp_dir);

        let tmp_file = tmp_dir.path().join(FILENAME_STDIN);

        let mut writer = BufWriter::new(File::create(&tmp_file)?);
        let mut buf = Vec::new();

        debug!("Reading contents from stdin...");
        while reader.read_until(b'\n', &mut buf)? != 0 {
            writer.write_all(buf.as_ref())?;
            buf.clear();
        }
        writer.flush()?;
        debug!("Contents from stdin stored to {:?}", tmp_file);

        match tmp_file.to_str() {
            Some(p) if tmp_file.exists() => self.build(p, Some(tmp_dir)),
            Some(p) => Err(Error::FileNotFoundError(p.to_string())),
            None => Err(Error::FilePathError(tmp_file.to_string_lossy().to_string())),
        }
    }

    fn build(&self, path: &str, temp_dir: Option<TempDir>) -> Result<Reader> {
        if let Some(p) = Self::tbi_path(path) {
            if !p.exists() {
                Err(Error::IndexNotFoundError(p.to_string_lossy().to_string()))?;
            }
        }

        let p = CString::new(path)?;
        let tbx: *mut htslib::tbx_t = unsafe { htslib::tbx_index_load(p.as_ptr()) };

        if tbx.is_null() {
            Err(htslib_error::Fetch)?;
        }

        let info = self.info(path);
        let info_keys = match self.info_keys.as_ref() {
            Some(vec) => vec.clone(),
            None => info.iter().map(|(k, _)| k.to_owned()).collect(),
        };

        Ok(Reader {
            reader: rust_htslib::bcf::Reader::from_path(path)?,
            references: self.references(path),
            filters: self.filters(path),
            info,
            info_keys,
            normalize: self.normalize,
            tbx,
            temp_dir,
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

    fn references(&self, path: &str) -> BTreeMap<u32, Sequence> {
        let mut map = BTreeMap::new();

        if let Ok(reader) = rust_htslib::bcf::Reader::from_path(path) {
            reader.header().header_records().iter().for_each(|x| {
                if let bcf::HeaderRecord::Contig { key: _key, values } = x {
                    if let Some(Ok(idx)) = values.get("IDX").map(|v| u32::from_str(v)) {
                        if let Some(id) = values.get("ID") {
                            if let Some(Some(seq)) = self.references.get(id) {
                                map.insert(idx, seq.clone());
                            }
                        }
                    }
                }
            });
        }

        map
    }

    fn filters(&self, path: &str) -> BTreeMap<u32, String> {
        let mut map = BTreeMap::new();

        if let Ok(reader) = rust_htslib::bcf::Reader::from_path(path) {
            reader.header().header_records().iter().for_each(|x| {
                if let bcf::HeaderRecord::Filter { values, .. } = x {
                    if let Some(v) = values.get("ID") {
                        if let Ok(id) = reader.header().name_to_id(v.as_bytes()) {
                            map.insert(id.0, v.to_owned());
                        }
                    }
                }
            });
        }

        map
    }

    fn info(&self, path: &str) -> BTreeMap<String, (bcf::header::TagType, bcf::header::TagLength)> {
        let mut map = BTreeMap::new();

        if let Ok(reader) = rust_htslib::bcf::Reader::from_path(path) {
            reader.header().header_records().iter().for_each(|x| {
                if let bcf::HeaderRecord::Info { values, .. } = x {
                    if let Some(v) = values.get("ID") {
                        if let Ok((typ, len)) = reader.header().info_type(v.as_bytes()) {
                            map.insert(v.to_owned(), (typ, len));
                        }
                    }
                }
            });
        }

        map
    }
}

#[derive(Debug)]
pub struct Reader {
    reader: bcf::Reader,
    // mapping contigs to references
    references: BTreeMap<u32, Sequence>,
    // header cache
    filters: BTreeMap<u32, String>,
    // header cache
    info: BTreeMap<String, (bcf::header::TagType, bcf::header::TagLength)>,
    // list of keys to read
    info_keys: Vec<String>,
    normalize: bool,
    tbx: *mut htslib::tbx_t,
    temp_dir: Option<TempDir>,
}

impl Reader {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        ReaderBuilder::new().path(path)
    }

    pub fn header(&self) -> &bcf::header::HeaderView {
        self.reader.header()
    }

    pub fn contigs(&self) -> BTreeMap<u32, String> {
        let mut map = BTreeMap::new();

        self.reader.header().header_records().iter().for_each(|x| {
            if let bcf::HeaderRecord::Contig { key: _key, values } = x {
                if let Some(v) = values.get("ID") {
                    if let Ok(i) = self.reader.header().name2rid(v.as_bytes()) {
                        map.insert(i, v.to_owned());
                    }
                }
            }
        });

        map
    }

    pub fn references(&self) -> &BTreeMap<u32, Sequence> {
        &self.references
    }

    pub fn info(&self) -> &BTreeMap<String, (bcf::header::TagType, bcf::header::TagLength)> {
        &self.info
    }

    pub fn info_keys(&self) -> &Vec<String> {
        &self.info_keys
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

    pub fn records(&mut self) -> Records<'_> {
        Records {
            reader: &mut self.reader,
            references: &self.references,
            filters: &self.filters,
            info: &self.info,
            info_keys: &self.info_keys,
            normalize: self.normalize,
        }
    }
}

impl Drop for Reader {
    fn drop(&mut self) {
        unsafe {
            htslib::tbx_destroy(self.tbx);
        }
    }
}

pub struct Records<'a> {
    reader: &'a mut bcf::Reader,
    references: &'a BTreeMap<u32, Sequence>,
    filters: &'a BTreeMap<u32, String>,
    info: &'a BTreeMap<String, (bcf::header::TagType, bcf::header::TagLength)>,
    info_keys: &'a Vec<String>,
    normalize: bool,
}

impl<'a> Iterator for Records<'a> {
    type Item = Result<record::Record<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut record = self.reader.empty_record();
        match self.reader.read(&mut record) {
            Some(Err(e)) => Some(Err(e.into())),
            Some(Ok(_)) => Some(Ok(record::Record::new(
                record,
                self.references,
                self.filters,
                self.info,
                self.info_keys,
                self.normalize,
            ))),
            None => None,
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

        assert!(keys.contains(&"RS".to_string()));
        assert!(keys.contains(&"CLNACC".to_string()));
        assert_eq!(keys.len(), 31);
    }

    #[test]
    fn test_info_types() {
        let vcf = Reader::from_path("test/dbsnp_example.vcf.gz").expect("Error opening file.");
        let info_types = vcf.info();

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
        let contigs = vcf.contigs();

        assert_eq!(contigs.get(&0).unwrap(), "NC_000001.10");
        assert_eq!(contigs.get(&23).unwrap(), "NC_000024.9");
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

    #[test]
    fn test_read_missing_reference() {
        let records = read_vcf_as_vec("test/visc_spec.vcf.gz");

        assert_eq!(records[3].alleles(), vec![b".", b"T"]);
    }

    #[test]
    fn test_read_missing_alternate() {
        let records = read_vcf_as_vec("test/visc_spec.vcf.gz");

        assert_eq!(records[5].alleles(), vec![b"T", b"."]);
        assert_eq!(records[6].alleles(), vec![b"T"]);
    }
}
