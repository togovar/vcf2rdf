//! Executable for converting VCF to RDF.
use structopt::StructOpt;

use std::collections::HashMap;
use std::convert::TryFrom;
use vcf2rdf::cli::configuration::Configuration;
use vcf2rdf::cli::CLI;
use vcf2rdf::errors::{Error, Result};
use vcf2rdf::rdf::namespace::Namespace;
use vcf2rdf::rdf::turtle_writer::TurtleWriter;
use vcf2rdf::rdf::writer::Writer;
use vcf2rdf::util::vcf;
use vcf2rdf::vcf::alteration::Alteration;
use vcf2rdf::vcf::assembly::{Assembly, Sequence};
use vcf2rdf::vcf::info::InfoKeys;
use vcf2rdf::vcf::reader::{Info, Reader};
use vcf2rdf::vcf::record::Record;

fn main() -> Result<()> {
    let options: CLI = CLI::from_args();

    let config = options
        .config
        .and_then(|ref p| Configuration::from_path(p).ok());

    let ns: Namespace = match config {
        Some(ref c) => Namespace::from(c),
        None => Namespace::default(),
    };

    let mut writer = TurtleWriter::new(std::io::stdout());
    writer.namespace(&ns);
    writer.subject_id(options.subject_id);

    let mut vcf = Reader::from_path(options.input)?;

    let contig = vcf.contig();

    let assembly = Assembly::try_from(options.assembly.as_str())?;
    let sequences: HashMap<u32, Sequence> = assembly.map_from(&contig);

    let filters = vcf.filters();
    let info_keys: InfoKeys = match &config {
        Some(c) => InfoKeys::from(c),
        None => InfoKeys::from(&vcf.info_keys()),
    };
    let info_types = vcf.info_types(&info_keys);

    for r in vcf.records() {
        let record = r?;

        let rid = record.rid().ok_or(Error::ReferenceIndexError)?;

        let seq = sequences
            .get(&rid)
            .ok_or(Error::ReferenceSequenceNotFoundError(rid))?;

        let filter: Vec<&str> = record
            .filters()
            .into_iter()
            .filter_map(|x| filters.get(&x.0).and_then(|x| Some(x.as_str())))
            .collect();

        let info: Vec<Info> = vcf::extract_info(&record, &info_keys, &info_types);

        for (i, a) in Alteration::from_record(&record).iter().enumerate() {
            if let Ok(alt) = a {
                writer.write_record(&Record {
                    inner: &record,
                    sequence: &seq,
                    alteration: &alt,
                    filter: &filter,
                    info: &info,
                    allele_index: i,
                })?
            }
        }

        if options.rehearsal {
            break;
        }
    }

    Ok(())
}
