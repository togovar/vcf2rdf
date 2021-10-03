use rust_htslib::bcf::Record;
use serde::Serialize;

use crate::errors::{Error, Result};

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Serialize)]
pub enum AlterationPosition {
    SNV(i64),
    Deletion(i64, i64),
    Insertion(i64, i64),
    Indel(i64, i64),
    MNV(i64, i64),
}

impl AlterationPosition {
    pub fn to_str(&self) -> &str {
        match self {
            AlterationPosition::SNV(_) => "SNV",
            AlterationPosition::Deletion(_, _) => "Deletion",
            AlterationPosition::Insertion(_, _) => "Insertion",
            AlterationPosition::Indel(_, _) => "Indel",
            AlterationPosition::MNV(_, _) => "MNV",
        }
    }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Serialize)]
pub struct Alteration<'a> {
    #[deprecated(note = "use normalized_position, instead.")]
    pub position: i64,
    #[deprecated(note = "use normalized_reference, instead.")]
    pub reference: &'a str,
    #[deprecated(note = "use normalized_alternate, instead.")]
    pub alternate: &'a str,
    pub normalized_position: AlterationPosition,
    pub normalized_reference: &'a str,
    pub normalized_alternate: &'a str,
}

impl<'a> Alteration<'a> {
    pub fn from_record(record: &'a Record) -> Vec<Result<Alteration>> {
        let mut alleles = record.alleles();

        let reference = alleles[0];

        if alleles.len() == 1 {
            alleles.push(b".");
        }

        alleles
            .iter()
            .skip(1)
            .map(|&x| {
                let position = record.pos() + 1;
                let reference = unsafe { std::str::from_utf8_unchecked(reference) };
                let alternate = unsafe { std::str::from_utf8_unchecked(x) };

                match Alteration::normalize(position, reference, alternate) {
                    Ok((n_pos, n_ref, n_alt)) => Ok(Alteration {
                        position,
                        reference,
                        alternate,
                        normalized_position: n_pos,
                        normalized_reference: n_ref,
                        normalized_alternate: n_alt,
                    }),
                    Err(e) => Err(e),
                }
            })
            .collect()
    }

    pub fn normalize(
        pos: i64,
        reference: &'a str,
        alternate: &'a str,
    ) -> Result<(AlterationPosition, &'a str, &'a str)> {
        let reference = match reference {
            "." => "",
            x => x,
        };
        let alternate = match alternate {
            "." => "",
            x => x,
        };

        let (reference, alternate, n) = Self::remove_shared_nucleotide(reference, alternate);

        match (reference.len(), alternate.len()) {
            (0, 0) => Err(Error::InvalidRefAltError)?,
            (1, 1) => Ok((AlterationPosition::SNV(pos + n), reference, alternate)),
            (0, _) => Ok((
                AlterationPosition::Insertion(pos + n - 1, pos + n),
                reference,
                alternate,
            )),
            (_, 0) => Ok((
                AlterationPosition::Deletion(pos + n, pos + n + reference.len() as i64 - 1),
                reference,
                alternate,
            )),
            (r, a) if r == a => Ok((
                AlterationPosition::MNV(pos + n, pos + n + reference.len() as i64 - 1),
                reference,
                alternate,
            )),
            (_, _) => Ok((
                AlterationPosition::Indel(pos + n, pos + n + reference.len() as i64 - 1),
                reference,
                alternate,
            )),
        }
    }

    fn remove_shared_nucleotide(reference: &'a str, alternate: &'a str) -> (&'a str, &'a str, i64) {
        let prefix = reference
            .chars()
            .zip(alternate.chars())
            .take_while(|(r, a)| r == a)
            .map(|x| x.0)
            .collect::<String>();

        match (
            reference.strip_prefix(&prefix),
            alternate.strip_prefix(&prefix),
        ) {
            (Some(r), Some(a)) => (r, a, prefix.len() as i64),
            _ => (reference, alternate, 0),
        }
    }
}

#[cfg(test)]
mod tests {
    use rust_htslib::bcf::{IndexedReader, Read};

    use super::*;

    #[test]
    fn test_normalize_snv() {
        let (n_pos, n_ref, n_alt) =
            Alteration::normalize(1000, "T", "C").expect("Error normalizing alteration.");

        assert_eq!(n_pos, AlterationPosition::SNV(1000));
        assert_eq!(n_ref, "T");
        assert_eq!(n_alt, "C");

        let (n_pos, n_ref, n_alt) =
            Alteration::normalize(1000, "AT", "AC").expect("Error normalizing alteration.");

        assert_eq!(n_pos, AlterationPosition::SNV(1001));
        assert_eq!(n_ref, "T");
        assert_eq!(n_alt, "C");
    }

    #[test]
    fn test_normalize_mnv() {
        let (n_pos, n_ref, n_alt) =
            Alteration::normalize(1000, "CT", "AG").expect("Error normalizing alteration.");

        assert_eq!(n_pos, AlterationPosition::MNV(1000, 1001));
        assert_eq!(n_ref, "CT");
        assert_eq!(n_alt, "AG");

        let (n_pos, n_ref, n_alt) =
            Alteration::normalize(1000, "ACT", "AAG").expect("Error normalizing alteration.");

        assert_eq!(n_pos, AlterationPosition::MNV(1001, 1002));
        assert_eq!(n_ref, "CT");
        assert_eq!(n_alt, "AG");
    }

    #[test]
    fn test_normalize_deletion() {
        let (n_pos, n_ref, n_alt) =
            Alteration::normalize(1000, "CT", "C").expect("Error normalizing alteration.");

        assert_eq!(n_pos, AlterationPosition::Deletion(1001, 1001));
        assert_eq!(n_ref, "T");
        assert_eq!(n_alt, "");

        let (n_pos, n_ref, n_alt) =
            Alteration::normalize(1001, "T", "").expect("Error normalizing alteration.");

        assert_eq!(n_pos, AlterationPosition::Deletion(1001, 1001));
        assert_eq!(n_ref, "T");
        assert_eq!(n_alt, "");

        let (n_pos, n_ref, n_alt) =
            Alteration::normalize(1001, "T", ".").expect("Error normalizing alteration.");

        assert_eq!(n_pos, AlterationPosition::Deletion(1001, 1001));
        assert_eq!(n_ref, "T");
        assert_eq!(n_alt, "");

        let (n_pos, n_ref, n_alt) =
            Alteration::normalize(1000, "ACTC", "AC").expect("Error normalizing alteration.");

        assert_eq!(n_pos, AlterationPosition::Deletion(1002, 1003));
        assert_eq!(n_ref, "TC");
        assert_eq!(n_alt, "");
    }

    #[test]
    fn test_normalize_insertion() {
        let (n_pos, n_ref, n_alt) =
            Alteration::normalize(1000, "C", "CT").expect("Error normalizing alteration.");

        assert_eq!(n_pos, AlterationPosition::Insertion(1000, 1001));
        assert_eq!(n_ref, "");
        assert_eq!(n_alt, "T");

        let (n_pos, n_ref, n_alt) =
            Alteration::normalize(1001, "", "T").expect("Error normalizing alteration.");

        assert_eq!(n_pos, AlterationPosition::Insertion(1000, 1001));
        assert_eq!(n_ref, "");
        assert_eq!(n_alt, "T");

        let (n_pos, n_ref, n_alt) =
            Alteration::normalize(1001, ".", "T").expect("Error normalizing alteration.");

        assert_eq!(n_pos, AlterationPosition::Insertion(1000, 1001));
        assert_eq!(n_ref, "");
        assert_eq!(n_alt, "T");

        let (n_pos, n_ref, n_alt) =
            Alteration::normalize(1000, "AC", "ACTG").expect("Error normalizing alteration.");

        assert_eq!(n_pos, AlterationPosition::Insertion(1001, 1002));
        assert_eq!(n_ref, "");
        assert_eq!(n_alt, "TG");
    }

    #[test]
    fn test_normalize_indel() {
        let (n_pos, n_ref, n_alt) =
            Alteration::normalize(1000, "CG", "CAT").expect("Error normalizing alteration.");

        assert_eq!(n_pos, AlterationPosition::Indel(1001, 1001));
        assert_eq!(n_ref, "G");
        assert_eq!(n_alt, "AT");

        let (n_pos, n_ref, n_alt) =
            Alteration::normalize(1000, "ACGT", "ACATA").expect("Error normalizing alteration.");

        assert_eq!(n_pos, AlterationPosition::Indel(1002, 1003));
        assert_eq!(n_ref, "GT");
        assert_eq!(n_alt, "ATA");
    }

    #[test]
    fn test_alteration() {
        let mut bcf =
            IndexedReader::from_path(&"test/dbsnp_example.vcf.gz").expect("Error opening file.");

        let rid = bcf
            .header()
            .name2rid(b"NC_000002.11")
            .expect("Translating from contig 'NC_000002.11' to ID failed.");

        bcf.fetch(rid, 10_018, 10_019).expect("Fetching failed");

        for r in bcf
            .records()
            .filter(|x| x.as_ref().unwrap().pos() == 10_018)
        {
            let record = r.as_ref().unwrap();
            let alterations = Alteration::from_record(&record);

            match alterations[0].as_ref() {
                Ok(alt) => {
                    assert_eq!(alt.reference, "C");
                    assert_eq!(alt.alternate, "CACA");
                }
                _ => unreachable!(),
            }
            match alterations[1].as_ref() {
                Ok(alt) => {
                    assert_eq!(alt.reference, "C");
                    assert_eq!(alt.alternate, "CACG");
                }
                _ => unreachable!(),
            }
        }
    }
}
