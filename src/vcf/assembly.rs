use once_cell::sync::Lazy;

#[derive(Debug, Clone)]
pub struct Sequence<'a> {
    pub name: &'a str,
    pub genbank: &'a str,
    pub refseq: &'a str,
    pub ucsc_name: &'a str,
    pub reference: &'a str,
}

#[derive(Debug, Clone)]
pub struct Assembly<'a> {
    name: &'a str,
    genbank: &'a str,
    refseq: &'a str,
    sequences: Vec<Sequence<'a>>,
}

impl<'a> Assembly<'a> {
    pub fn find_sequence(&self, name: &String) -> Option<&Sequence> {
        self.sequences.iter().find(|&x| {
            x.name == name || x.genbank == name || x.refseq == name || x.ucsc_name == name
        })
    }
}

impl<'a> From<&crate::cli::generator::Assembly> for Assembly<'a> {
    fn from(assembly: &crate::cli::generator::Assembly) -> Self {
        match assembly {
            crate::cli::generator::Assembly::GRCH37 => GRCH37_P13.clone(),
            crate::cli::generator::Assembly::GRCH38 => GRCH38_P13.clone(),
            crate::cli::generator::Assembly::GRCM38 => GRCM38.clone(),
            crate::cli::generator::Assembly::GRCM39 => GRCM39.clone(),
        }
    }
}

macro_rules! sequences {
    (
        $(
            ($name:expr, $genbank:expr, $refseq:expr, $ucsc_name:expr, $reference:expr);
        )+
    ) => {
        vec![
        $(
            Sequence {
                name: $name,
                genbank: $genbank,
                refseq: $refseq,
                ucsc_name: $ucsc_name,
                reference: $reference,
            },
        )+
        ]
    };
}

static GRCH37_P13: Lazy<Assembly> = Lazy::new(|| Assembly {
    name: "GRCh37",
    genbank: "GCA_000001405.14",
    refseq: "GCF_000001405.25",
    sequences: sequences! {
        ("1", "CM000663.1", "NC_000001.10", "chr1", "http://identifiers.org/hco/1/GRCh37");
        ("2", "CM000664.1", "NC_000002.11", "chr2", "http://identifiers.org/hco/2/GRCh37");
        ("3", "CM000665.1", "NC_000003.11", "chr3", "http://identifiers.org/hco/3/GRCh37");
        ("4", "CM000666.1", "NC_000004.11", "chr4", "http://identifiers.org/hco/4/GRCh37");
        ("5", "CM000667.1", "NC_000005.9", "chr5", "http://identifiers.org/hco/5/GRCh37");
        ("6", "CM000668.1", "NC_000006.11", "chr6", "http://identifiers.org/hco/6/GRCh37");
        ("7", "CM000669.1", "NC_000007.13", "chr7", "http://identifiers.org/hco/7/GRCh37");
        ("8", "CM000670.1", "NC_000008.10", "chr8", "http://identifiers.org/hco/8/GRCh37");
        ("9", "CM000671.1", "NC_000009.11", "chr9", "http://identifiers.org/hco/9/GRCh37");
        ("10", "CM000672.1", "NC_000010.10", "chr10", "http://identifiers.org/hco/10/GRCh37");
        ("11", "CM000673.1", "NC_000011.9", "chr11", "http://identifiers.org/hco/11/GRCh37");
        ("12", "CM000674.1", "NC_000012.11", "chr12", "http://identifiers.org/hco/12/GRCh37");
        ("13", "CM000675.1", "NC_000013.10", "chr13", "http://identifiers.org/hco/13/GRCh37");
        ("14", "CM000676.1", "NC_000014.8", "chr14", "http://identifiers.org/hco/14/GRCh37");
        ("15", "CM000677.1", "NC_000015.9", "chr15", "http://identifiers.org/hco/15/GRCh37");
        ("16", "CM000678.1", "NC_000016.9", "chr16", "http://identifiers.org/hco/16/GRCh37");
        ("17", "CM000679.1", "NC_000017.10", "chr17", "http://identifiers.org/hco/17/GRCh37");
        ("18", "CM000680.1", "NC_000018.9", "chr18", "http://identifiers.org/hco/18/GRCh37");
        ("19", "CM000681.1", "NC_000019.9", "chr19", "http://identifiers.org/hco/19/GRCh37");
        ("20", "CM000682.1", "NC_000020.10", "chr20", "http://identifiers.org/hco/20/GRCh37");
        ("21", "CM000683.1", "NC_000021.8", "chr21", "http://identifiers.org/hco/21/GRCh37");
        ("22", "CM000684.1", "NC_000022.10", "chr22", "http://identifiers.org/hco/22/GRCh37");
        ("X", "CM000685.1", "NC_000023.10", "chrX", "http://identifiers.org/hco/X/GRCh37");
        ("Y", "CM000686.1", "NC_000024.9", "chrY", "http://identifiers.org/hco/Y/GRCh37");
        ("MT", "J01415.2", "NC_012920.1", "chrM", "http://identifiers.org/hco/MT/GRCh37");
    },
});

static GRCH38_P13: Lazy<Assembly> = Lazy::new(|| Assembly {
    name: "GRCh38",
    genbank: "GCA_000001405.28",
    refseq: "GCF_000001405.39",
    sequences: sequences! {
        ("1", "CM000663.2", "NC_000001.11", "chr1", "http://identifiers.org/hco/1/GRCh38");
        ("2", "CM000664.2", "NC_000002.12", "chr2", "http://identifiers.org/hco/2/GRCh38");
        ("3", "CM000665.2", "NC_000003.12", "chr3", "http://identifiers.org/hco/3/GRCh38");
        ("4", "CM000666.2", "NC_000004.12", "chr4", "http://identifiers.org/hco/4/GRCh38");
        ("5", "CM000667.2", "NC_000005.10", "chr5", "http://identifiers.org/hco/5/GRCh38");
        ("6", "CM000668.2", "NC_000006.12", "chr6", "http://identifiers.org/hco/6/GRCh38");
        ("7", "CM000669.2", "NC_000007.14", "chr7", "http://identifiers.org/hco/7/GRCh38");
        ("8", "CM000670.2", "NC_000008.11", "chr8", "http://identifiers.org/hco/8/GRCh38");
        ("9", "CM000671.2", "NC_000009.12", "chr9", "http://identifiers.org/hco/9/GRCh38");
        ("10", "CM000672.2", "NC_000010.11", "chr10", "http://identifiers.org/hco/10/GRCh38");
        ("11", "CM000673.2", "NC_000011.10", "chr11", "http://identifiers.org/hco/11/GRCh38");
        ("12", "CM000674.2", "NC_000012.12", "chr12", "http://identifiers.org/hco/12/GRCh38");
        ("13", "CM000675.2", "NC_000013.11", "chr13", "http://identifiers.org/hco/13/GRCh38");
        ("14", "CM000676.2", "NC_000014.9", "chr14", "http://identifiers.org/hco/14/GRCh38");
        ("15", "CM000677.2", "NC_000015.10", "chr15", "http://identifiers.org/hco/15/GRCh38");
        ("16", "CM000678.2", "NC_000016.10", "chr16", "http://identifiers.org/hco/16/GRCh38");
        ("17", "CM000679.2", "NC_000017.11", "chr17", "http://identifiers.org/hco/17/GRCh38");
        ("18", "CM000680.2", "NC_000018.10", "chr18", "http://identifiers.org/hco/18/GRCh38");
        ("19", "CM000681.2", "NC_000019.10", "chr19", "http://identifiers.org/hco/19/GRCh38");
        ("20", "CM000682.2", "NC_000020.11", "chr20", "http://identifiers.org/hco/20/GRCh38");
        ("21", "CM000683.2", "NC_000021.9", "chr21", "http://identifiers.org/hco/21/GRCh38");
        ("22", "CM000684.2", "NC_000022.11", "chr22", "http://identifiers.org/hco/22/GRCh38");
        ("X", "CM000685.2", "NC_000023.11", "chrX", "http://identifiers.org/hco/X/GRCh38");
        ("Y", "CM000686.2", "NC_000024.10", "chrY", "http://identifiers.org/hco/Y/GRCh38");
        ("MT", "J01415.2", "NC_012920.1", "chrM", "http://identifiers.org/hco/MT/GRCh38");
    },
});

static GRCM38: Lazy<Assembly> = Lazy::new(|| Assembly {
    name: "GRCm38",
    genbank: "GCA_000001635.2",
    refseq: "GCF_000001635.20",
    sequences: sequences! {
        ("1", "CM000994.2", "NC_000067.6", "chr1", "https://identifiers.org/refseq/NC_000067.6");
        ("2", "CM000995.2", "NC_000068.7", "chr2", "https://identifiers.org/refseq/NC_000068.7");
        ("3", "CM000996.2", "NC_000069.6", "chr3", "https://identifiers.org/refseq/NC_000069.6");
        ("4", "CM000997.2", "NC_000070.6", "chr4", "https://identifiers.org/refseq/NC_000070.6");
        ("5", "CM000998.2", "NC_000071.6", "chr5", "https://identifiers.org/refseq/NC_000071.6");
        ("6", "CM000999.2", "NC_000072.6", "chr6", "https://identifiers.org/refseq/NC_000072.6");
        ("7", "CM001000.2", "NC_000073.6", "chr7", "https://identifiers.org/refseq/NC_000073.6");
        ("8", "CM001001.2", "NC_000074.6", "chr8", "https://identifiers.org/refseq/NC_000074.6");
        ("9", "CM001002.2", "NC_000075.6", "chr9", "https://identifiers.org/refseq/NC_000075.6");
        ("10", "CM001003.2", "NC_000076.6", "chr10", "https://identifiers.org/refseq/NC_000076.6");
        ("11", "CM001004.2", "NC_000077.6", "chr11", "https://identifiers.org/refseq/NC_000077.6");
        ("12", "CM001005.2", "NC_000078.6", "chr12", "https://identifiers.org/refseq/NC_000078.6");
        ("13", "CM001006.2", "NC_000079.6", "chr13", "https://identifiers.org/refseq/NC_000079.6");
        ("14", "CM001007.2", "NC_000080.6", "chr14", "https://identifiers.org/refseq/NC_000080.6");
        ("15", "CM001008.2", "NC_000081.6", "chr15", "https://identifiers.org/refseq/NC_000081.6");
        ("16", "CM001009.2", "NC_000082.6", "chr16", "https://identifiers.org/refseq/NC_000082.6");
        ("17", "CM001010.2", "NC_000083.6", "chr17", "https://identifiers.org/refseq/NC_000083.6");
        ("18", "CM001011.2", "NC_000084.6", "chr18", "https://identifiers.org/refseq/NC_000084.6");
        ("19", "CM001012.2", "NC_000085.6", "chr19", "https://identifiers.org/refseq/NC_000085.6");
        ("X", "CM001013.2", "NC_000086.7", "chrX", "https://identifiers.org/refseq/NC_000086.7");
        ("Y", "CM001014.2", "NC_000087.7", "chrY", "https://identifiers.org/refseq/NC_000087.7");
    },
});

static GRCM39: Lazy<Assembly> = Lazy::new(|| Assembly {
    name: "GRCm39",
    genbank: "GCA_000001635.9",
    refseq: "GCF_000001635.27",
    sequences: sequences! {
        ("1", "CM000994.3", "NC_000067.7", "chr1", "https://identifiers.org/refseq/NC_000067.7");
        ("2", "CM000995.3", "NC_000068.8", "chr2", "https://identifiers.org/refseq/NC_000068.8");
        ("3", "CM000996.3", "NC_000069.7", "chr3", "https://identifiers.org/refseq/NC_000069.7");
        ("4", "CM000997.3", "NC_000070.7", "chr4", "https://identifiers.org/refseq/NC_000070.7");
        ("5", "CM000998.3", "NC_000071.7", "chr5", "https://identifiers.org/refseq/NC_000071.7");
        ("6", "CM000999.3", "NC_000072.7", "chr6", "https://identifiers.org/refseq/NC_000072.7");
        ("7", "CM001000.3", "NC_000073.7", "chr7", "https://identifiers.org/refseq/NC_000073.7");
        ("8", "CM001001.3", "NC_000074.7", "chr8", "https://identifiers.org/refseq/NC_000074.7");
        ("9", "CM001002.3", "NC_000075.7", "chr9", "https://identifiers.org/refseq/NC_000075.7");
        ("10", "CM001003.3", "NC_000076.7", "chr10", "https://identifiers.org/refseq/NC_000076.7");
        ("11", "CM001004.3", "NC_000077.7", "chr11", "https://identifiers.org/refseq/NC_000077.7");
        ("12", "CM001005.3", "NC_000078.7", "chr12", "https://identifiers.org/refseq/NC_000078.7");
        ("13", "CM001006.3", "NC_000079.7", "chr13", "https://identifiers.org/refseq/NC_000079.7");
        ("14", "CM001007.3", "NC_000080.7", "chr14", "https://identifiers.org/refseq/NC_000080.7");
        ("15", "CM001008.3", "NC_000081.7", "chr15", "https://identifiers.org/refseq/NC_000081.7");
        ("16", "CM001009.3", "NC_000082.7", "chr16", "https://identifiers.org/refseq/NC_000082.7");
        ("17", "CM001010.3", "NC_000083.7", "chr17", "https://identifiers.org/refseq/NC_000083.7");
        ("18", "CM001011.3", "NC_000084.7", "chr18", "https://identifiers.org/refseq/NC_000084.7");
        ("19", "CM001012.3", "NC_000085.7", "chr19", "https://identifiers.org/refseq/NC_000085.7");
        ("X", "CM001013.3", "NC_000086.8", "chrX", "https://identifiers.org/refseq/NC_000086.8");
        ("Y", "CM001014.3", "NC_000087.8", "chrY", "https://identifiers.org/refseq/NC_000087.8");
    },
});
