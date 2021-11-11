use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::cli::Subject;
use crate::errors::Result;
use crate::rdf::namespace::Namespace;
use crate::rdf::writer::Writer;
use crate::vcf::record::Record;

pub trait AsTurtle<W> {
    fn as_ttl_string(&self, wtr: &TurtleWriter<W>) -> Result<Option<String>>
    where
        W: Write;
}

#[derive(Debug)]
pub struct TurtleWriter<'a, W: Write> {
    wtr: BufWriter<W>,
    state: WriterState,
    namespace: Option<&'a Namespace>,
    info_key: Option<&'a Vec<String>>,
    pub subject_id: Option<Subject>,
}

#[derive(Debug)]
enum HeaderState {
    DidNotWrite,
    DidWrite,
}

#[derive(Debug)]
struct WriterState {
    header: HeaderState,
}

impl<'a> TurtleWriter<'a, File> {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<TurtleWriter<'a, File>> {
        Ok(Self::new(File::create(path)?))
    }
}

impl<'a, W: Write> TurtleWriter<'a, W> {
    pub fn new(wtr: W) -> TurtleWriter<'a, W> {
        TurtleWriter {
            wtr: BufWriter::new(wtr),
            state: WriterState {
                header: HeaderState::DidNotWrite,
            },
            namespace: None,
            info_key: None,
            subject_id: None,
        }
    }

    pub fn namespace(&mut self, namespace: &'a Namespace) -> &TurtleWriter<'a, W> {
        self.namespace = Some(namespace);
        self
    }

    pub fn info_key(&mut self, info_key: Option<&'a Vec<String>>) -> &TurtleWriter<'a, W> {
        self.info_key = info_key;
        self
    }

    pub fn subject(&mut self, subject_id: Option<Subject>) -> &TurtleWriter<'a, W> {
        self.subject_id = subject_id;
        self
    }

    fn write_headers(&mut self) -> Result<()> {
        let mut buf = String::with_capacity(4096);

        let max_len = self
            .namespace
            .unwrap()
            .prefixes
            .keys()
            .max_by_key(|x| x.len())
            .unwrap()
            .len();

        if let Some(ref base) = self.namespace.unwrap().base {
            buf += &format!("@base {:>width$}<{}> .\n", "", base, width = max_len + 4);
        }

        for (k, v) in &self.namespace.unwrap().prefixes {
            buf += &format!("@prefix {:>width$}: <{}> .\n", k, v, width = max_len);
        }

        buf += "\n";

        Ok(self.wtr.write_all(buf.as_bytes())?)
    }
}

impl<'a, W: Write> Writer for TurtleWriter<'a, W> {
    fn write_record<'b>(&mut self, record: &Record<'b>) -> Result<()> {
        if let HeaderState::DidNotWrite = self.state.header {
            self.write_headers()?;
            self.state.header = HeaderState::DidWrite;
        }

        if let Some(r) = record.as_ttl_string(&self)? {
            self.wtr.write_all(r.as_bytes())?;
        }

        Ok(())
    }
}
