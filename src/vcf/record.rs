use rust_htslib::bcf::Record as VCFRecord;

use crate::vcf::alteration::Alteration;
use crate::vcf::assembly::Sequence;
use crate::vcf::reader::Info;

pub mod as_turtle;

#[derive(Debug)]
pub struct Record<'a> {
    pub inner: &'a VCFRecord,
    pub sequence: &'a Sequence<'a>,
    pub alteration: &'a Alteration<'a>,
    pub filter: &'a Vec<&'a str>,
    pub info: &'a Vec<Info<'a>>,
    pub allele_index: usize,
}
