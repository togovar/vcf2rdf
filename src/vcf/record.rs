use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};

use rust_htslib::bcf;

use crate::config::Sequence;
use crate::errors;

pub mod as_turtle;

#[derive(Debug, Eq, PartialEq)]
pub struct Contig {
    pub rid: u32,
    pub name: String,
}

#[derive(Debug, PartialEq)]
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

#[derive(Debug)]
pub struct Record<'a> {
    inner: bcf::record::Record,
    references: &'a BTreeMap<u32, Sequence>,
    filters: &'a BTreeMap<u32, String>,
    info: &'a BTreeMap<String, (bcf::header::TagType, bcf::header::TagLength)>,
    info_keys: &'a Vec<String>,
    normalize: bool,
}

impl<'a> Record<'a> {
    pub fn new(
        inner: bcf::record::Record,
        references: &'a BTreeMap<u32, Sequence>,
        filters: &'a BTreeMap<u32, String>,
        info: &'a BTreeMap<String, (bcf::header::TagType, bcf::header::TagLength)>,
        info_keys: &'a Vec<String>,
        normalize: bool,
    ) -> Self {
        Self {
            inner,
            references,
            filters,
            info,
            info_keys,
            normalize,
        }
    }

    pub fn inner(&self) -> &bcf::record::Record {
        &self.inner
    }

    pub fn sequence(&self) -> Option<&Sequence> {
        if let Some(rid) = self.inner.rid() {
            self.references.get(&rid)
        } else {
            None
        }
    }

    pub fn chromosome(&self) -> Option<errors::Result<&str>> {
        self.inner.rid().map(|x| {
            Ok(self
                .inner
                .header()
                .rid2name(x)
                .map(|x| unsafe { std::str::from_utf8_unchecked(x) })?)
        })
    }

    pub fn quality(&self) -> f32 {
        self.inner.qual()
    }

    pub fn filters(&self) -> Vec<&str> {
        self.inner
            .filters()
            .into_iter()
            .filter_map(|x| self.filters.get(&x.0).map(|x| x.as_str()))
            .collect()
    }

    pub fn info(&self) -> Vec<Info> {
        self.info_keys
            .iter()
            .filter_map(|key| match self.info.get(key.to_string().as_str()) {
                Some(&(typ, length)) => match typ {
                    bcf::header::TagType::Flag => self.extract_flag(key),
                    bcf::header::TagType::Integer => self.extract_integer(key),
                    bcf::header::TagType::Float => self.extract_float(key),
                    bcf::header::TagType::String => self.extract_string(key),
                }
                .and_then(|value| {
                    Some(Info {
                        key,
                        value,
                        typ,
                        length,
                    })
                }),
                _ => self.extract_string(key).and_then(|value| {
                    Some(Info {
                        key,
                        value,
                        typ: bcf::header::TagType::String,
                        length: bcf::header::TagLength::Fixed(1),
                    })
                }),
            })
            .collect()
    }

    fn extract_string<S: AsRef<str>>(&self, key: S) -> Option<Vec<InfoValue>> {
        let info = self.inner.info(key.as_ref().as_bytes());

        info.string().ok().and_then(|string| {
            string.and_then(|v| {
                Some(
                    v.iter()
                        .map(|&x| unsafe {
                            InfoValue::String(String::from_utf8_unchecked(x.to_vec()))
                        })
                        .collect(),
                )
            })
        })
    }

    fn extract_float<S: AsRef<str>>(&self, key: S) -> Option<Vec<InfoValue>> {
        let info = self.inner.info(key.as_ref().as_bytes());

        info.float().ok().and_then(|float| {
            float.and_then(|v| Some(v.iter().map(|&x| InfoValue::Float(x)).collect()))
        })
    }

    fn extract_integer<S: AsRef<str>>(&self, key: S) -> Option<Vec<InfoValue>> {
        let info = self.inner.info(key.as_ref().as_bytes());

        info.integer().ok().and_then(|integer| {
            integer.and_then(|v| Some(v.iter().map(|&x| InfoValue::Integer(x)).collect()))
        })
    }

    fn extract_flag<S: AsRef<str>>(&self, key: S) -> Option<Vec<InfoValue>> {
        let mut info = self.inner.info(key.as_ref().as_bytes());

        info.flag()
            .ok()
            .and_then(|flag| Some(vec![InfoValue::Flag(flag)]))
    }

    pub fn each_alternate_alleles(&self) -> Entries {
        Entries {
            record: self,
            index: 0,
        }
    }
}

#[derive(Debug)]
pub struct Entries<'a> {
    record: &'a Record<'a>,
    index: usize,
}

impl<'a> Iterator for Entries<'a> {
    type Item = Entry<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self
            .record
            .inner
            .alleles()
            .get(self.index + 1)
            .map(|&x| Entry {
                record: self.record,
                alternate_allele: x,
                index: self.index,
            });

        self.index += 1;

        item
    }
}

#[derive(Debug)]
pub struct Entry<'a> {
    record: &'a Record<'a>,
    alternate_allele: &'a [u8],
    index: usize,
}

impl<'a> Display for Entry<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "chrom: {:?}, pos: {}, ref: {:?}, alt: {:?}",
            self.chromosome()
                .and_then(|x| Some(x.unwrap_or("?")))
                .unwrap_or("?"),
            self.position(),
            self.reference_bases(),
            self.alternate_bases(),
        )
    }
}

impl<'a> Entry<'a> {
    pub fn record(&self) -> &Record {
        self.record
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn chromosome(&self) -> Option<errors::Result<&str>> {
        self.record.inner().rid().map(|x| {
            Ok(self
                .record
                .inner()
                .header()
                .rid2name(x)
                .map(|x| unsafe { std::str::from_utf8_unchecked(x) })?)
        })
    }

    // `bcf::record::Record.pos()` returns 0-based position
    pub fn position(&self) -> u64 {
        self.record.inner().pos() as u64 + 1
    }

    pub fn id(&self) -> Option<String> {
        match unsafe { std::str::from_utf8_unchecked(self.record.inner().id().as_slice()) } {
            "." => None,
            v if v.is_empty() => None,
            v => Some(v.to_owned()),
        }
    }

    pub fn reference_bases(&self) -> &str {
        self.record
            .inner()
            .alleles()
            .first()
            .map_or("", |&x| unsafe { std::str::from_utf8_unchecked(x) })
    }

    pub fn alternate_bases(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(self.alternate_allele) }
    }
}
