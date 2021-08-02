use std::io::Write;

use rust_htslib::bcf;

use crate::cli::Subject;
use crate::errors::Result;
use crate::rdf::turtle_writer::{AsTurtle, TurtleWriter};
use crate::vcf::alteration::AlterationPosition;
use crate::vcf::reader::InfoValue;
use crate::vcf::record::Record;

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

impl<W: Write> AsTurtle<W> for Record<'_> {
    fn as_ttl_string(&self, wtr: &TurtleWriter<W>) -> Result<Option<String>> {
        if self.sequence.reference.is_none() {
            return Ok(None);
        }

        let mut buf = Buffer::default();

        let id = unsafe { String::from_utf8_unchecked(self.inner.id()) };
        let alt = self.alteration;
        let qual = self.inner.qual();

        self.write_subject(&wtr, &mut buf);

        buf.push_str(" a gvo:");
        buf.push_str(self.alteration.normalized_position.to_str());

        buf.push_str(" ;\n  dct:identifier ");
        buf.push_quoted(&id, '"');

        self.write_location(&mut buf);

        buf.push_str(" ;\n  gvo:ref ");
        buf.push_quoted(alt.normalized_reference, '"');

        buf.push_str(" ;\n  gvo:alt ");
        buf.push_quoted(alt.normalized_alternate, '"');

        if qual.is_finite() {
            buf.push_str(" ;\n  gvo:qual ");
            buf.push_str(qual.to_string().as_str());
        }

        let filters = self.filter;
        if filters.len() > 0 {
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

impl Record<'_> {
    fn write_subject<W: Write>(&self, wtr: &TurtleWriter<W>, buf: &mut Buffer) {
        match &wtr.subject_id {
            Some(Subject::ID) => {
                let id = unsafe { String::from_utf8_unchecked(self.inner.id()) };
                buf.push_iri(&id);
            }
            Some(Subject::Location) => {
                let alt = self.alteration;

                buf.push_str("<");
                if let Some(ref name) = self.sequence.name {
                    buf.push_str(name.as_str());
                    buf.push_str("-");
                }
                buf.push_str(alt.position.to_string().as_str());
                buf.push_str("-");
                buf.push_str(alt.reference);
                buf.push_str("-");
                buf.push_str(alt.alternate);
                buf.push_str(">");
            }
            _ => {
                buf.push_str("[]");
            }
        };
    }

    fn write_location(&self, buf: &mut Buffer) {
        buf.push_str(" ;\n  faldo:location [");
        match self.alteration.normalized_position {
            AlterationPosition::SNV(p) => {
                buf.push_str("\n    a faldo:ExactPosition ;");
                buf.push_str("\n    faldo:position ");
                buf.push_str(p.to_string().as_str());
                if let Some(ref reference) = self.sequence.reference {
                    buf.push_str(" ;\n    faldo:reference ");
                    buf.push_iri(reference.as_str());
                }
            }
            AlterationPosition::Deletion(begin, end) | AlterationPosition::Indel(begin, end) => {
                buf.push_str("\n    a faldo:Region ;");
                buf.push_str("\n    faldo:begin [");
                buf.push_str("\n      a faldo:InBetweenPosition ;");
                buf.push_str("\n      faldo:after ");
                buf.push_str((begin - 1).to_string().as_str());
                buf.push_str(" ;\n      faldo:before ");
                buf.push_str(begin.to_string().as_str());
                if let Some(ref reference) = self.sequence.reference {
                    buf.push_str(" ;\n    faldo:reference ");
                    buf.push_iri(reference.as_str());
                }
                buf.push_str("\n    ] ;");

                buf.push_str("\n    faldo:end [");
                buf.push_str("\n      a faldo:InBetweenPosition ;");
                buf.push_str("\n      faldo:after ");
                buf.push_str(end.to_string().as_str());
                buf.push_str(" ;\n      faldo:before ");
                buf.push_str((end + 1).to_string().as_str());
                if let Some(ref reference) = self.sequence.reference {
                    buf.push_str(" ;\n    faldo:reference ");
                    buf.push_iri(reference.as_str());
                }
                buf.push_str("\n    ]");
            }
            AlterationPosition::Insertion(begin, end) => {
                buf.push_str("\n    a faldo:InBetweenPosition ;");
                buf.push_str("\n    faldo:after ");
                buf.push_str(begin.to_string().as_str());
                buf.push_str(" ;\n    faldo:before ");
                buf.push_str(end.to_string().as_str());
                if let Some(ref reference) = self.sequence.reference {
                    buf.push_str(" ;\n    faldo:reference ");
                    buf.push_iri(reference.as_str());
                }
            }
            AlterationPosition::MNV(begin, end) => {
                buf.push_str("\n    a faldo:Region ;");
                buf.push_str("\n    faldo:begin ");
                buf.push_str(begin.to_string().as_str());
                buf.push_str(" ;\n    faldo:end ");
                buf.push_str(end.to_string().as_str());
                if let Some(ref reference) = self.sequence.reference {
                    buf.push_str(" ;\n    faldo:reference ");
                    buf.push_iri(reference.as_str());
                }
            }
        };
        buf.push_str("\n  ]");
    }

    fn write_info(&self, buf: &mut Buffer) {
        let info = self.info;
        if info.len() > 0 {
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
                            if i == self.allele_index {
                                self.push_info_value(buf, v);
                            }
                        }
                    }
                    (vs, bcf::header::TagLength::Alleles) => {
                        let r = &vs.get(0);
                        let a = &vs.get(self.allele_index + 1);

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
        } else {
            buf.push_str(" ;\n  gvo:info []");
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
                buf.push_quoted(str, '"');
            }
        };
    }
}
