use crate::cli::configuration::Configuration;
use crate::cli::Generate;
use crate::errors::Result;
use crate::vcf::reader::Reader;

pub fn run(command: Generate) -> Result<()> {
    match command {
        Generate::Config { input } => {
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
