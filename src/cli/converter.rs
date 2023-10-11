use std::path::PathBuf;

use structopt::StructOpt;
use strum::{EnumString, EnumVariantNames, VariantNames};

use crate::config::Config;
use crate::errors::Result;
use crate::rdf::namespace::Namespace;
use crate::rdf::turtle_writer::{SubjectFormatter, TurtleWriter};
use crate::rdf::writer::Writer;
use crate::vcf::reader::ReaderBuilder;

#[derive(EnumString, EnumVariantNames, Debug)]
#[strum(serialize_all = "snake_case")]
pub enum Subject {
    ID,
    Location,
    Reference,
    NormalizedLocation,
    NormalizedReference,
}

#[derive(StructOpt, Debug)]
pub struct Options {
    /// Path to configuration yaml.
    #[structopt(short, long, parse(from_os_str))]
    pub config: PathBuf,

    /// Processes only one record and exit.
    #[structopt(long)]
    pub rehearsal: bool,

    /// Do not normalize faldo representation.
    #[structopt(long)]
    pub no_normalize: bool,

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

    let mut builder = ReaderBuilder::new()
        .reference(config.reference)
        .normalize(!options.no_normalize);

    if let Some(keys) = config.info {
        builder = builder.info_keys(keys);
    }

    let mut reader = builder.path(options.input)?;

    for record in reader.records() {
        let record = record?;

        writer.write_record(&record)?;

        if options.rehearsal {
            break;
        }
    }

    Ok(())
}
