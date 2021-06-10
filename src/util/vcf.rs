//! Module for utility functions with VCF
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

use rust_htslib::bcf;
use rust_htslib::htslib;

use crate::errors::Result;
use crate::vcf::info::InfoKeys;
use crate::vcf::reader::{Info, InfoValue};

pub mod compress;

/// Returns the hts format information
///
/// Definition of `htsFormat` and its constants are:
/// ```ignore
/// #[repr(C)]
/// #[derive(Debug, Copy, Clone)]
/// pub struct htsFormat {
///     pub category: htsFormatCategory,
///     pub format: htsExactFormat,
///     pub version: htsFormat__bindgen_ty_1,
///     pub compression: htsCompression,
///     pub compression_level: ::std::os::raw::c_short,
///     pub specific: *mut ::std::os::raw::c_void,
/// }
///
/// #[repr(C)]
/// #[derive(Debug, Copy, Clone)]
/// pub struct htsFormat__bindgen_ty_1 {
///     pub major: ::std::os::raw::c_short,
///     pub minor: ::std::os::raw::c_short,
/// }
///
/// pub const htsFormatCategory_unknown_category: htsFormatCategory = 0;
/// pub const htsFormatCategory_sequence_data: htsFormatCategory = 1;
/// pub const htsFormatCategory_variant_data: htsFormatCategory = 2;
/// pub const htsFormatCategory_index_file: htsFormatCategory = 3;
/// pub const htsFormatCategory_region_list: htsFormatCategory = 4;
/// pub const htsFormatCategory_category_maximum: htsFormatCategory = 32767;
/// pub type htsFormatCategory = u32;
///
/// pub const htsExactFormat_unknown_format: htsExactFormat = 0;
/// pub const htsExactFormat_binary_format: htsExactFormat = 1;
/// pub const htsExactFormat_text_format: htsExactFormat = 2;
/// pub const htsExactFormat_sam: htsExactFormat = 3;
/// pub const htsExactFormat_bam: htsExactFormat = 4;
/// pub const htsExactFormat_bai: htsExactFormat = 5;
/// pub const htsExactFormat_cram: htsExactFormat = 6;
/// pub const htsExactFormat_crai: htsExactFormat = 7;
/// pub const htsExactFormat_vcf: htsExactFormat = 8;
/// pub const htsExactFormat_bcf: htsExactFormat = 9;
/// pub const htsExactFormat_csi: htsExactFormat = 10;
/// pub const htsExactFormat_gzi: htsExactFormat = 11;
/// pub const htsExactFormat_tbi: htsExactFormat = 12;
/// pub const htsExactFormat_bed: htsExactFormat = 13;
/// pub const htsExactFormat_htsget: htsExactFormat = 14;
/// pub const htsExactFormat_json: htsExactFormat = 14;
/// pub const htsExactFormat_empty_format: htsExactFormat = 15;
/// pub const htsExactFormat_fasta_format: htsExactFormat = 16;
/// pub const htsExactFormat_fastq_format: htsExactFormat = 17;
/// pub const htsExactFormat_fai_format: htsExactFormat = 18;
/// pub const htsExactFormat_fqi_format: htsExactFormat = 19;
/// pub const htsExactFormat_hts_crypt4gh_format: htsExactFormat = 20;
/// pub const htsExactFormat_format_maximum: htsExactFormat = 32767;
/// pub type htsExactFormat = u32;
///
/// pub const htsCompression_no_compression: htsCompression = 0;
/// pub const htsCompression_gzip: htsCompression = 1;
/// pub const htsCompression_bgzf: htsCompression = 2;
/// pub const htsCompression_custom: htsCompression = 3;
/// pub const htsCompression_bzip2_compression: htsCompression = 4;
/// pub const htsCompression_compression_maximum: htsCompression = 32767;
/// pub type htsCompression = u32;
/// ```
pub fn get_format<P: AsRef<Path>>(path: P) -> Result<htslib::htsFormat> {
    let format = unsafe {
        let bcf: Reader = std::mem::transmute(bcf::Reader::from_path(&path)?);
        *htslib::hts_get_format(bcf.inner)
    };

    Ok(format)
}

#[allow(dead_code)]
struct Reader {
    inner: *mut htslib::htsFile,
    header: Rc<bcf::header::HeaderView>,
}

impl Drop for Reader {
    fn drop(&mut self) {
        unsafe {
            htslib::hts_close(self.inner);
        }
    }
}

// TODO: implement in Record
pub fn extract_info<'a>(
    record: &'a bcf::record::Record,
    info_keys: &'a InfoKeys,
    info_types: &'a HashMap<String, (bcf::header::TagType, bcf::header::TagLength)>,
) -> Vec<Info<'a>> {
    info_keys
        .iter()
        .filter_map(|key| match info_types.get(key) {
            Some(&(typ, length)) => match typ {
                bcf::header::TagType::Flag => extract_flag(record, key),
                bcf::header::TagType::Integer => extract_integer(record, key),
                bcf::header::TagType::Float => extract_float(record, key),
                bcf::header::TagType::String => extract_string(record, key),
            }
            .and_then(|value| {
                Some(Info {
                    key: key.as_str(),
                    value,
                    typ,
                    length,
                })
            }),
            _ => extract_string(record, key).and_then(|value| {
                Some(Info {
                    key: key.as_str(),
                    value,
                    typ: bcf::header::TagType::String,
                    length: bcf::header::TagLength::Fixed(1),
                })
            }),
        })
        .collect()
}

fn extract_string(record: &bcf::record::Record, key: &String) -> Option<Vec<InfoValue>> {
    let info = record.info(key.as_bytes());

    info.string().ok().and_then(|string| {
        string.and_then(|v| {
            Some(
                v.iter()
                    .map(|&x| unsafe { InfoValue::String(String::from_utf8_unchecked(x.to_vec())) })
                    .collect(),
            )
        })
    })
}

fn extract_float(record: &bcf::record::Record, key: &String) -> Option<Vec<InfoValue>> {
    let info = record.info(key.as_bytes());

    info.float().ok().and_then(|float| {
        float.and_then(|v| Some(v.iter().map(|&x| InfoValue::Float(x)).collect()))
    })
}

fn extract_integer(record: &bcf::record::Record, key: &String) -> Option<Vec<InfoValue>> {
    let info = record.info(key.as_bytes());

    info.integer().ok().and_then(|integer| {
        integer.and_then(|v| Some(v.iter().map(|&x| InfoValue::Integer(x)).collect()))
    })
}

fn extract_flag(record: &bcf::record::Record, key: &String) -> Option<Vec<InfoValue>> {
    let mut info = record.info(key.as_bytes());

    info.flag()
        .ok()
        .and_then(|flag| Some(vec![InfoValue::Flag(flag)]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_format() {
        let format = get_format("test/dbsnp_example.vcf").unwrap();

        assert_eq!(format.category, 2);
        assert_eq!(format.format, 8);
        assert_eq!(format.version.major, 4);
        assert_eq!(format.version.minor, 2);
        assert_eq!(format.compression, 0);
    }

    #[test]
    fn test_get_format_bgzf() {
        let format = get_format("test/dbsnp_example.vcf.gz").unwrap();

        assert_eq!(format.category, 2);
        assert_eq!(format.format, 8);
        assert_eq!(format.version.major, 4);
        assert_eq!(format.version.minor, 2);
        assert_eq!(format.compression, 2);
    }
}
