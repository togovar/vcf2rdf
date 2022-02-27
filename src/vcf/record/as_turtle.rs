use std::io::Write;

use rust_htslib::bcf;

use crate::errors::Result;
use crate::rdf::turtle_writer::{AsTurtle, TurtleWriter};
use crate::rdf::writer::Writer;
use crate::vcf::record::{Entry, InfoValue};

const BUFFER_DEFAULT: usize = 40 * 1024;

struct Buffer {
    string: String,
}

impl Default for Buffer {
    fn default() -> Self {
        Buffer {
            string: String::with_capacity(BUFFER_DEFAULT),
        }
    }
}

impl Buffer {
    pub fn push_str(&mut self, string: &str) {
        self.string.push_str(string)
    }

    pub fn push_iri(&mut self, string: &str) {
        self.string.push('<');
        self.string.push_str(string);
        self.string.push('>')
    }

    pub fn push_quoted(&mut self, string: &str, quote: char) -> () {
        self.string.push(quote);
        self.string.push_str(string);
        self.string.push(quote);
    }
}

impl<W: Write> AsTurtle<W> for Entry<'_> {
    fn as_ttl_string(&self, wtr: &TurtleWriter<W>) -> Result<Option<String>>
    where
        W: Write,
    {
        let mut buf = Buffer::default();

        match wtr.format_subject(&self) {
            Some(v) => {
                buf.push_str("<");
                buf.push_str(v.as_str());
                buf.push_str(">");
            }
            None => buf.push_str("[]"),
        }

        let (pos, reference, alternate, vc) = self.normalize();
        let (pos, reference, alternate) = if self.record.normalize {
            (pos, reference, alternate)
        } else {
            (
                self.position(),
                self.reference_bases(),
                self.alternate_bases(),
            )
        };

        if let Some(v) = vc.as_ref() {
            buf.push_str(" a gvo:");
            buf.push_str(v.to_str());
        } else {
            buf.push_str(" a gvo:Variation");
        };

        let id = unsafe { String::from_utf8_unchecked(self.record.inner.id()) };
        if !id.is_empty() || id != "." {
            buf.push_str(" ;\n  dct:identifier ");
            buf.push_quoted(&id, '"');
        }

        self.write_location(&mut buf, pos, reference, alternate);

        buf.push_str(" ;\n  gvo:ref ");
        buf.push_quoted(reference.unwrap_or(""), '"');

        buf.push_str(" ;\n  gvo:alt ");
        buf.push_quoted(alternate.unwrap_or(""), '"');

        let quality = self.record.quality();
        if quality.is_finite() {
            buf.push_str(" ;\n  gvo:qual ");
            buf.push_str(quality.to_string().as_str());
        }

        let filters = self.record.filters();
        if !filters.is_empty() {
            buf.push_str(" ;\n  gvo:filter ");

            for (i, filter) in filters.iter().enumerate() {
                if i != 0 {
                    buf.push_str(", ");
                };
                buf.push_quoted(filter, '"');
            }
        }

        self.write_info(&mut buf);

        buf.push_str(" .\n\n");

        Ok(Some(buf.string))
    }
}

impl Entry<'_> {
    fn write_location(
        &self,
        buf: &mut Buffer,
        position: i64,
        reference: Option<&str>,
        alternate: Option<&str>,
    ) {
        let seq = self.record.sequence().map(|x| x.reference.as_ref());

        buf.push_str(" ;\n  faldo:location [");

        match (
            reference.map_or(0, |v| v.len()),
            alternate.map_or(0, |v| v.len()),
        ) {
            (1, 1) => {
                // SNV
                buf.push_str("\n    a faldo:ExactPosition ;");
                buf.push_str("\n    faldo:position ");
                buf.push_str(position.to_string().as_str());
                if let Some(Some(seq)) = seq {
                    buf.push_str(" ;\n    faldo:reference ");
                    buf.push_iri(seq);
                }
            }
            (0, _) => {
                // Insertion
                buf.push_str("\n    a faldo:InBetweenPosition ;");
                buf.push_str("\n    faldo:after ");
                buf.push_str((position - 1).to_string().as_str());
                buf.push_str(" ;\n    faldo:before ");
                buf.push_str(position.to_string().as_str());
                if let Some(Some(seq)) = seq {
                    buf.push_str(" ;\n    faldo:reference ");
                    buf.push_iri(seq);
                }
            }
            (r, a) if r == a => {
                // MNV
                let p1 = position;
                let p2 = position + reference.map_or(0, |v| v.len() as i64 - 1);
                buf.push_str("\n    a faldo:Region ;");
                buf.push_str("\n    faldo:begin ");
                buf.push_str(p1.to_string().as_str());
                buf.push_str(" ;\n    faldo:end ");
                buf.push_str(p2.to_string().as_str());
                if let Some(Some(seq)) = seq {
                    buf.push_str(" ;\n    faldo:reference ");
                    buf.push_iri(seq);
                }
            }
            (_, _) => {
                // Deletion, Indel
                let p1 = position;
                let p2 = position + reference.map_or(0, |v| v.len() as i64 - 1);
                buf.push_str("\n    a faldo:Region ;");
                buf.push_str("\n    faldo:begin [");
                buf.push_str("\n      a faldo:InBetweenPosition ;");
                buf.push_str("\n      faldo:after ");
                buf.push_str((p1 - 1).to_string().as_str());
                buf.push_str(" ;\n      faldo:before ");
                buf.push_str(p1.to_string().as_str());
                if let Some(Some(seq)) = seq {
                    buf.push_str(" ;\n      faldo:reference ");
                    buf.push_iri(seq);
                }
                buf.push_str("\n    ] ;");

                buf.push_str("\n    faldo:end [");
                buf.push_str("\n      a faldo:InBetweenPosition ;");
                buf.push_str("\n      faldo:after ");
                buf.push_str(p2.to_string().as_str());
                buf.push_str(" ;\n      faldo:before ");
                buf.push_str((p2 + 1).to_string().as_str());
                if let Some(Some(seq)) = seq {
                    buf.push_str(" ;\n      faldo:reference ");
                    buf.push_iri(seq);
                }
                buf.push_str("\n    ]");
            }
        };

        buf.push_str("\n  ]");
    }

    fn write_info(&self, buf: &mut Buffer) {
        let info = self.record.info();
        if !info.is_empty() {
            buf.push_str(" ;\n  gvo:info");

            for (i, info) in info.iter().enumerate() {
                buf.push_str(if i == 0 { " [" } else { ", [" });
                buf.push_str("\n    rdfs:label ");
                buf.push_quoted(info.key, '"');
                buf.push_str(" ;\n    rdf:value ");

                match (&info.value, &info.length) {
                    (vs, bcf::header::TagLength::Fixed(n)) => {
                        let n = match &info.typ {
                            bcf::header::TagType::Flag => 1,
                            _ => *n,
                        };
                        for (i, v) in vs.iter().take(n as usize).enumerate() {
                            if i != 0 {
                                buf.push_str(", ");
                            };
                            self.push_info_value(buf, v);
                        }
                    }
                    (vs, bcf::header::TagLength::AltAlleles) => {
                        for (i, v) in vs.iter().enumerate() {
                            if i == self.index {
                                self.push_info_value(buf, v);
                            }
                        }
                    }
                    (vs, bcf::header::TagLength::Alleles) => {
                        let r = &vs.get(0);
                        let a = &vs.get(self.index + 1);

                        match (&r, &a) {
                            (Some(r), Some(a)) => {
                                buf.push_quoted(format!("{},{}", r, a).as_str(), '"')
                            }
                            _ => panic!("failed to obtain value"),
                        }
                        buf.push_str(" ;\n    rdf:comment \"This field contains two values, the first is the value for the reference allele and the second is the value for the alternate allele.\"");
                    }
                    (vs, len) => {
                        for (i, v) in vs.iter().enumerate() {
                            if i != 0 {
                                buf.push_str(", ");
                            };
                            self.push_info_value(buf, v);
                        }

                        if len == &bcf::header::TagLength::Genotypes {
                            buf.push_str(" ;\n    rdf:comment \"The field has one value for each possible genotype.\"");
                        }
                    }
                }

                buf.push_str("\n  ]");
            }
        }
    }

    fn push_info_value(&self, buf: &mut Buffer, v: &InfoValue) {
        match v {
            InfoValue::Flag(x) => {
                buf.push_str(x.to_string().as_str());
            }
            InfoValue::Integer(x) => {
                buf.push_str(x.to_string().as_str());
            }
            InfoValue::Float(x) => {
                buf.push_str(x.to_string().as_str());
            }
            InfoValue::String(str) => {
                if str.contains("%") {
                    buf.push_quoted(Self::percent_decode(str).as_str(), '"');
                } else {
                    buf.push_quoted(str, '"');
                }
            }
        };
    }

    fn percent_decode<T: AsRef<str>>(str: T) -> String {
        str.as_ref()
            .replace("%3A", ":")
            .replace("%3B", ";")
            .replace("%3D", "=")
            .replace("%25", "%")
            .replace("%2C", ",")
            .replace("%0D", "\r")
            .replace("%0A", "\n")
            .replace("%09", "\t")
    }
}
