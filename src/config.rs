use std::collections::BTreeMap;
use std::fs::File;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::errors::Result;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Sequence {
    pub name: Option<String>,
    pub reference: Option<String>,
}

/// A structure for user configuration.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub base: Option<String>,
    pub namespaces: Option<BTreeMap<String, String>>,
    pub info: Option<Vec<String>>,
    pub reference: BTreeMap<String, Option<Sequence>>,
}

impl Config {
    /// Read a yaml configuration from a given path.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Config> {
        Ok(serde_yaml::from_reader(File::open(path)?)?)
    }
}
