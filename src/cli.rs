//! Module for command line interface
use std::path::PathBuf;

use structopt::clap::crate_description;
use structopt::StructOpt;
use strum::{EnumString, EnumVariantNames, VariantNames};

pub mod configuration;
pub mod converter;
pub mod generator;
pub mod statistics;

#[derive(StructOpt, Debug)]
#[structopt(about = crate_description!())]
pub enum Command {
    /// Converts VCF to RDF.
    Convert(Convert),

    /// Prints statistics.
    Stat(Statistics),

    /// Generates template.
    Generate(Generate),
}

#[derive(EnumString, EnumVariantNames, Debug)]
#[strum(serialize_all = "lowercase")]
pub enum Subject {
    ID,
    Location,
}

#[derive(StructOpt, Debug)]
pub struct Convert {
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

#[derive(StructOpt, Debug)]
pub enum Statistics {
    /// Counts records.
    Count {
        /// Path to file to process.
        #[structopt(name = "FILE", parse(from_os_str))]
        input: PathBuf,
    },
}

#[derive(EnumString, EnumVariantNames, Debug)]
pub enum Assembly {
    #[strum(serialize = "GRCh37")]
    GRCH37,
    #[strum(serialize = "GRCh38")]
    GRCH38,
    #[strum(serialize = "GRCm38")]
    GRCM38,
    #[strum(serialize = "GRCm39")]
    GRCM39,
}

#[derive(StructOpt, Debug)]
pub enum Generate {
    /// Generates config template.
    Config {
        /// Pre-defined assembly.
        #[structopt(short, long, possible_values = Assembly::VARIANTS)]
        assembly: Option<Assembly>,

        /// Path to file to process.
        #[structopt(name = "FILE", parse(from_os_str))]
        input: PathBuf,
    },
}
