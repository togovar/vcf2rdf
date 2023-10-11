use std::collections::BTreeMap;
use std::path::PathBuf;

use structopt::StructOpt;
use strum::VariantNames;
use strum::{EnumString, EnumVariantNames};

use crate::config::{Config, Sequence};
use crate::errors::Result;
use crate::vcf::assembly::{GRCH37_P13, GRCH38_P13, GRCM38, GRCM39};
use crate::vcf::reader::Reader;

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
pub enum Options {
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

pub fn run(command: Options) -> Result<()> {
    match command {
        Options::Config { assembly, input } => {
            let vcf = Reader::from_path(input)?;

            let assembly = match assembly.as_ref() {
                Some(v) => match v {
                    Assembly::GRCH37 => Some(GRCH37_P13.clone()),
                    Assembly::GRCH38 => Some(GRCH38_P13.clone()),
                    Assembly::GRCM38 => Some(GRCM38.clone()),
                    Assembly::GRCM39 => Some(GRCM39.clone()),
                },
                None => None,
            };

            let mut reference = BTreeMap::new();
            for (_, name) in vcf.contigs().iter() {
                // TODO: M -> MT
                let seq = assembly
                    .as_ref()
                    .map(|x| {
                        x.find_sequence(name).map(|x| Sequence {
                            name: Some(String::from(x.name)),
                            reference: Some(String::from(x.reference)),
                        })
                    })
                    .unwrap_or(None);

                reference.insert(name.to_owned(), seq.or(Some(Sequence::default())));
            }

            let config = Config {
                base: None,
                namespaces: None,
                info: Some(vcf.info_keys().clone()),
                reference,
            };

            let mut yaml = serde_yaml::to_string(&config)?;

            if let Some(i) = yaml.find("base:") {
                yaml.insert_str(i, "\n# Set base IRI if needed.\n");
            }

            if let Some(i) = yaml.find("namespaces:") {
                yaml.insert_str(i, "\n# Additional namespaces.\n");
            }

            if let Some(i) = yaml.find("reference:") {
                yaml.insert_str(i, "\n# Sequence reference mapping.\n");
            }

            if let Some(i) = yaml.find("info:") {
                yaml.insert_str(i, "\n# Remove unnecessary keys to convert.\n");
            }

            println!("{}", &yaml);
        }
    }

    Ok(())
}
