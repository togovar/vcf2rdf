use std::fs::File;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::errors::Result;
use crate::rdf::namespace::Namespaces;

/// A structure for user configuration.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Configuration {
    pub base: Option<String>,
    pub namespaces: Option<Namespaces>,
    pub info: Option<Vec<String>>,
}

impl Configuration {
    /// Read a yaml configuration from a given path.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(serde_yaml::from_reader(File::open(path)?)?)
    }
}
