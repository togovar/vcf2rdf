//! Module for command line interface
use structopt::clap::crate_description;
use structopt::StructOpt;

pub mod compressor;
pub mod converter;
pub mod generator;
pub mod statistics;

#[derive(StructOpt, Debug)]
#[structopt(about = crate_description!())]
pub enum Command {
    /// Compress VCF to BGZF.
    Compress(compressor::Options),

    /// Converts VCF to RDF.
    Convert(converter::Options),

    /// Prints statistics.
    Stat(statistics::Options),

    /// Generates template.
    Generate(generator::Options),
}
