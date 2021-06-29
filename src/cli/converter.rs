use crate::cli::configuration::{Configuration, Sequence};
use crate::cli::Convert;
use crate::errors::{Error, Result};
use crate::rdf::namespace::Namespace;
use crate::rdf::turtle_writer::TurtleWriter;
use crate::rdf::writer::Writer;
use crate::util::vcf;
use crate::vcf::alteration::Alteration;
use crate::vcf::info::InfoKeys;
use crate::vcf::reader::{Info, Reader};
use crate::vcf::record::Record;

pub fn run(options: Convert) -> Result<()> {
    let config = Configuration::from_path(&options.config)?;

    let mut writer = TurtleWriter::new(std::io::stdout());

    let ns = Namespace::from(&config);
    writer.namespace(&ns);
    writer.subject(options.subject);

    let mut vcf = Reader::from_path(options.input)?;

    let sequences: Vec<Option<Sequence>> = {
        let contig = vcf.contig();
        let max: usize = contig
            .iter()
            .max_by(|x, y| x.rid.cmp(&y.rid))
            .map_or(0, |x| x.rid as usize);

        let mut vec = vec![None; max + 1];
        for c in contig {
            vec[c.rid as usize] = config.reference.get(&c.name).unwrap().to_owned();
        }

        vec
    };

    let filters = vcf.filters();
    let info_keys = InfoKeys::from(&config);
    let info_types = vcf.info_types(&info_keys);

    for r in vcf.records() {
        let record = r?;

        let rid = record.rid().ok_or(Error::ReferenceIndexError)?;

        let seq = sequences.get(rid as usize).unwrap().as_ref().unwrap();

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
                    sequence: seq,
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
