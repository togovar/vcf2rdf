use crate::errors::Result;
use crate::vcf::record::Record;

pub trait Writer {
    fn write_record(&mut self, record: &Record) -> Result<()>;
}
