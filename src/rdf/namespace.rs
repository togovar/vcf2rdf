use std::collections::BTreeMap;

use crate::config::Config;

const DCT: &str = "http://purl.org/dc/terms/";
const FALDO: &str = "http://biohackathon.org/resource/faldo#";
const GVO: &str = "http://genome-variation.org/resource#";
const HCO: &str = "http://identifiers.org/hco/";
const OBO: &str = "http://purl.obolibrary.org/obo/";
const RDF: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
const RDFS: &str = "http://www.w3.org/2000/01/rdf-schema#";
const SIO: &str = "http://semanticscience.org/resource/";

/// RDF namespace.
#[derive(Debug)]
pub struct Namespace {
    pub base: Option<String>,
    pub prefixes: BTreeMap<String, String>,
}

impl Default for Namespace {
    /// Default for `Namespace`
    fn default() -> Self {
        let mut prefixes = BTreeMap::new();

        prefixes.insert("dct".to_owned(), DCT.to_owned());
        prefixes.insert("faldo".to_owned(), FALDO.to_owned());
        prefixes.insert("gvo".to_owned(), GVO.to_owned());
        prefixes.insert("hco".to_owned(), HCO.to_owned());
        prefixes.insert("obo".to_owned(), OBO.to_owned());
        prefixes.insert("rdf".to_owned(), RDF.to_owned());
        prefixes.insert("rdfs".to_owned(), RDFS.to_owned());
        prefixes.insert("sio".to_owned(), SIO.to_owned());

        Namespace {
            base: None,
            prefixes,
        }
    }
}

impl From<&Config> for Namespace {
    /// Create from `cli::configuration::Configuration`
    fn from(cnf: &Config) -> Self {
        let mut ns = Namespace::default();

        ns.base = match cnf.base {
            Some(ref x) => Some(x.clone()),
            None => None,
        };

        // merge with default namespaces
        if let Some(ref x) = cnf.namespaces {
            ns.prefixes.extend(x.clone());
        }

        ns
    }
}
