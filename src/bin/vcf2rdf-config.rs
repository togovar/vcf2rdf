//! Executable for working with VCF2RDF configuration.
use structopt::StructOpt;

use vcf2rdf::cli::configuration::Configuration;
use vcf2rdf::cli::{VCF2RDFConfig, VCF2RDFConfigCommand};
use vcf2rdf::errors::Result;
use vcf2rdf::vcf::reader::Reader;

fn main() -> Result<()> {
    let options: VCF2RDFConfig = VCF2RDFConfig::from_args();

    match options.cmd {
        VCF2RDFConfigCommand::Generate { input } => {
            let vcf = Reader::from_path(input)?;

            let config = Configuration {
                base: None,
                namespaces: None,
                info: Some(vcf.info_keys()),
            };

            let mut yaml = serde_yaml::to_string(&config)?;

            if let Some(i) = yaml.find("base:") {
                yaml.insert_str(i, "\n# Set base IRI if needed.\n");
            }

            if let Some(i) = yaml.find("namespaces:") {
                yaml.insert_str(i, "\n# Additional namespaces.\n");
            }

            if let Some(i) = yaml.find("info:") {
                yaml.insert_str(i, "\n# Remove unnecessary keys to convert.\n");
            }

            println!("{}", &yaml);
        }
    }

    Ok(())
}
