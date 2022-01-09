use std::collections::BTreeMap;
use std::path::PathBuf;

use structopt::StructOpt;
use strum::{EnumString, EnumVariantNames, VariantNames};

use crate::config::Config;
use crate::errors::{Error, Result};
use crate::rdf::namespace::Namespace;
use crate::rdf::turtle_writer::{SubjectFormatter, TurtleWriter};
use crate::rdf::writer::Writer;
use crate::util::vcf;
use crate::vcf::alteration::Alteration;
use crate::vcf::info::InfoKeys;
use crate::vcf::reader::{Info, Reader};
use crate::vcf::record::Record;

#[derive(EnumString, EnumVariantNames, Debug)]
#[strum(serialize_all = "lowercase")]
pub enum Subject {
    ID,
    Location,
}

#[derive(StructOpt, Debug)]
pub struct Options {
    /// Path to configuration yaml.
    #[structopt(short, long, parse(from_os_str))]
    pub config: PathBuf,

    /// Processes only one record and exit.
    #[structopt(long)]
    pub rehearsal: bool,

    /// Strategy to generate a subject (use blank node if not specified).
    /// If use `id`, ensure that all values at ID column are present and unique.
    #[structopt(short, long, possible_values = Subject::VARIANTS)]
    pub subject: Option<Subject>,

    /// Path to file to process.
    #[structopt(parse(from_os_str))]
    pub input: PathBuf,
}

pub fn run(options: Options) -> Result<()> {
    let config = Config::from_path(options.config)?;

    let mut writer = TurtleWriter::new(std::io::stdout());

    let ns = Namespace::from(&config);
    writer.namespace(&ns);

    if let Some(v) = options.subject.as_ref() {
        writer.subject_formatter(SubjectFormatter::from(v));
    }

    let mut vcf = Reader::from_path(options.input)?;

    let sequences = {
        let contig = vcf.contig();

        let mut map = BTreeMap::new();

        for c in contig {
            map.insert(
                c.rid,
                config
                    .reference
                    .get(&c.name)
                    .ok_or(Error::ConfigurationNotFoundError(c.name.to_owned()))?
                    .as_ref()
                    .ok_or(Error::InvalidConfigurationError(c.name.to_owned()))?,
            );
        }

        map
    };

    let filters = vcf.filters();
    let info_keys = InfoKeys::from(&config);
    let info_types = vcf.info_types(&info_keys);

    for r in vcf.records() {
        let record = r?;

        let rid = record.rid().ok_or(Error::ReferenceIndexError)?;

        let seq = *sequences.get(&rid).unwrap();

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
