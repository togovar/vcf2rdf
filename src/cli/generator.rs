use std::collections::BTreeMap;
use std::convert::TryFrom;

use crate::cli::configuration::{Configuration, Sequence};
use crate::cli::Generate;
use crate::errors::Result;
use crate::vcf::assembly::Assembly;
use crate::vcf::reader::Reader;

pub fn run(command: Generate) -> Result<()> {
    match command {
        Generate::Config { assembly, input } => {
            let vcf = Reader::from_path(input)?;

            let assembly = match assembly {
                Some(ref a) => Assembly::try_from(a).ok(),
                None => None,
            };

            let mut reference = BTreeMap::new();
            for c in vcf.contig().iter() {
                let seq = assembly
                    .as_ref()
                    .map(|x| {
                        x.find_sequence(&c.name).map(|x| Sequence {
                            name: Some(String::from(x.name)),
                            reference: Some(String::from(x.reference)),
                        })
                    })
                    .unwrap_or(None);

                reference.insert(c.name.to_owned(), seq.or(Some(Sequence::default())));
            }

            let config = Configuration {
                base: None,
                namespaces: None,
                info: Some(vcf.info_keys()),
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
