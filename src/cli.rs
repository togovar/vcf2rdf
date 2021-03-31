//! Module for command line interface
use std::path::PathBuf;

use structopt::clap::crate_description;
use structopt::StructOpt;
use strum::{EnumString, EnumVariantNames, VariantNames};

#[derive(EnumString, EnumVariantNames, Debug)]
#[strum(serialize_all = "kebab_case")]
pub enum OutputFormat {
    NTriples,
    Turtle,
}

#[derive(StructOpt, Debug)]
#[structopt(about = crate_description!())]
pub struct CLI {
    /// Verbose mode{n}
    /// -v   -> report statistics messages{n}
    /// -vv  -> report warning messages{n}
    /// -vvv -> report debug messages
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,

    /// Log device [default: STDERR]
    #[structopt(long, parse(from_os_str))]
    log_dev: Option<PathBuf>,

    /// Output format
    #[structopt(short, long, possible_values = OutputFormat::VARIANTS, case_insensitive = true, default_value = "n-triples")]
    format: OutputFormat,

    /// File to process
    #[structopt(name = "FILE", parse(from_os_str))]
    input: PathBuf,
}

#[derive(StructOpt, Debug)]
#[structopt(name = "vcf_stat", about = "Obtain VCF statistics")]
pub struct VCFStat {
    #[structopt(subcommand)]
    pub cmd: Command,
}

#[derive(StructOpt, Debug)]
pub enum Command {
    /// Count records
    Count {
        /// File to process
        #[structopt(name = "FILE", parse(from_os_str))]
        input: PathBuf,
    },
}
