use rust_htslib::bcf::Record as VCFRecord;

use crate::cli::configuration::Sequence;
use crate::vcf::alteration::Alteration;
use crate::vcf::reader::Info;

pub mod as_turtle;

#[derive(Debug)]
pub struct Record<'a> {
    pub inner: &'a VCFRecord,
    pub sequence: &'a Sequence,
    pub alteration: &'a Alteration<'a>,
    pub filter: &'a Vec<&'a str>,
    pub info: &'a Vec<Info<'a>>,
    pub allele_index: usize,
}
