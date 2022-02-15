use crate::errors::Result;
use crate::vcf::record::{Entry, Record};

pub trait Writer {
    fn write_record(&mut self, record: &Record) -> Result<()>;
    fn write_entry(&mut self, entry: &Entry) -> Result<()>;
    fn format_subject(&self, entry: &Entry) -> Option<String>;
}
