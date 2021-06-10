use std::collections::HashMap;

use crate::cli::configuration::Configuration;

const DCT: &str = "http://purl.org/dc/terms/";
const FALDO: &str = "http://biohackathon.org/resource/faldo#";
const GVO: &str = "http://genome-variation.org/resource#";
const HCO: &str = "http://identifiers.org/hco/";
const OBO: &str = "http://purl.obolibrary.org/obo/";
const RDF: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
const RDFS: &str = "http://www.w3.org/2000/01/rdf-schema#";
const SIO: &str = "http://semanticscience.org/resource/";

pub type Namespaces = HashMap<String, String>;

/// RDF namespace.
#[derive(Debug)]
pub struct Namespace {
    pub base: Option<String>,
    pub namespaces: Namespaces,
}

impl Default for Namespace {
    /// Default for `Namespace`
    fn default() -> Self {
        let mut m: HashMap<String, String> = HashMap::new();

        m.insert(String::from("dct"), DCT.to_string());
        m.insert(String::from("faldo"), FALDO.to_string());
        m.insert(String::from("gvo"), GVO.to_string());
        m.insert(String::from("hco"), HCO.to_string());
        m.insert(String::from("obo"), OBO.to_string());
        m.insert(String::from("rdf"), RDF.to_string());
        m.insert(String::from("rdfs"), RDFS.to_string());
        m.insert(String::from("sio"), SIO.to_string());

        Namespace {
            base: None,
            namespaces: m,
        }
    }
}

impl From<&Configuration> for Namespace {
    /// Create from `cli::configuration::Configuration`
    fn from(cnf: &Configuration) -> Self {
        let mut ns = Namespace::default();

        ns.base = match cnf.base {
            Some(ref x) => Some(x.clone()),
            None => None,
        };

        // merge with default namespaces
        if let Some(ref x) = cnf.namespaces {
            ns.namespaces.extend(x.clone());
        }

        ns
    }
}
