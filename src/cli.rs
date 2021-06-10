//! Module for command line interface
use std::path::PathBuf;

use structopt::clap::crate_description;
use structopt::StructOpt;
use strum::{EnumString, EnumVariantNames, VariantNames};

pub mod configuration;

#[derive(EnumString, EnumVariantNames, Debug)]
#[strum(serialize_all = "lowercase")]
pub enum SubjectID {
    ID,
    Location,
}

#[derive(StructOpt, Debug)]
#[structopt(about = crate_description!())]
pub struct CLI {
    /// Assembly. (e.g. GRCh37, GRCh38)
    #[structopt(short, long)]
    pub assembly: String, // TODO: Support user's definition

    /// Path to configuration yaml.
    #[structopt(short, long, parse(from_os_str))]
    pub config: Option<PathBuf>,

    /// Process only one record and exit.
    #[structopt(long)]
    pub rehearsal: bool,

    /// Strategy to generate subject ID (use blank node if not specified).
    /// If use `id`, ensure that all values at ID column are present and unique.
    #[structopt(short, long, possible_values = SubjectID::VARIANTS)]
    pub subject_id: Option<SubjectID>,

    /// Path to file to process.
    #[structopt(parse(from_os_str))]
    pub input: PathBuf,
}

#[derive(StructOpt, Debug)]
#[structopt(name = "vcf_stat", about = "Obtain VCF statistics")]
pub struct VCFStat {
    #[structopt(subcommand)]
    pub cmd: Command,
}

#[derive(StructOpt, Debug)]
pub enum Command {
    /// Count records.
    Count {
        /// File to process.
        #[structopt(name = "FILE", parse(from_os_str))]
        input: PathBuf,
    },
}
