use std::slice::Iter;

use crate::config::Config;

#[derive(Debug)]
pub struct InfoKeys {
    inner: Vec<String>,
}

impl InfoKeys {
    pub fn iter(&self) -> Iter<'_, String> {
        self.inner.iter()
    }
}

impl From<&Config> for InfoKeys {
    fn from(c: &Config) -> Self {
        InfoKeys {
            inner: match &c.info {
                Some(x) => x.clone(),
                None => vec![],
            },
        }
    }
}

impl From<&Vec<String>> for InfoKeys {
    fn from(v: &Vec<String>) -> Self {
        InfoKeys { inner: v.clone() }
    }
}
