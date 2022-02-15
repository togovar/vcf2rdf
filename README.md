# vcf2rdf

[![GitHub](https://img.shields.io/github/license/togovar/vcf2rdf)](https://github.com/togovar/vcf2rdf/blob/main/LICENSE)
[![GitHub release (latest SemVer)](https://img.shields.io/github/v/release/togovar/vcf2rdf?logo=github)](https://github.com/togovar/vcf2rdf/releases)
[![Docker Image Version (latest semver)](https://img.shields.io/docker/v/togovar/vcf2rdf?logo=docker)](https://hub.docker.com/r/togovar/vcf2rdf)

vcf2rdf is a command-line tool for converting from [Variant Call Format (VCF)](https://en.wikipedia.org/wiki/Variant_Call_Format) to [Resource Description Format (RDF)](https://en.wikipedia.org/wiki/Resource_Description_Framework).
Since vcf2rdf aims to convert large VCF fast, the generated RDF model is very simple.

## Installation

### Use docker

```shell
$ docker pull togoavr/vcf2rdf
```

### Use pre-built binary

Download the latest version from [releases page](https://github.com/togovar/vcf2rdf/releases).

### Manual build

#### Prerequisites

- [rust](https://www.rust-lang.org)
- cargo (automatically installed by `rustup`)

#### How to build

```shell
$ git clone https://github.com/togovar/vcf2rdf.git
$ cd vcf2rdf
$ cargo build --release
$ ./target/release/vcf2rdf --help
```

## Usage

```
USAGE:
    vcf2rdf <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    compress    Compress VCF to BGZF
    convert     Converts VCF to RDF
    generate    Generates template
    help        Prints this message or the help of the given subcommand(s)
    stat        Prints statistics
```

### Convert VCF to RDF

Currently, `vcf2rdf` can only handle gzipped and tabix-indexed vcf.
Compressing and indexing the VCF with `bgzip` and `tabix` is required.
For more details, visit [Samtools](https://www.htslib.org).

```shell
$ bgzip -c input.vcf > input.vcf.gz
$ tabix input.vcf.gz
```

To generate configuration template:

```shell
$ vcf2rdf generate config --assembly GRCh38 input.vcf.gz > config.yaml
```

then output:

```yaml
---

# Set base IRI if needed.
base: ~

# Additional namespaces.
namespaces: ~

# Remove unnecessary keys to convert.
info:
  - key1
  - key2
  - key3
  - ...

# Sequence reference mapping.
reference:
  "1":           # The value of CHROM column
    name: ~      # Used if --subject=location or --subject=normalized_location are passed to the converter
    reference: ~ # Used for URI of faldo:reference / if --subject=reference or --subject=normalized_reference are passed to the converter
  ...
```

All keys listed in VCF meta-information lines (##INFO=<...>) are filled in info.
Remove key names that you do not need in converted RDF.
Without a configuration file, all values in the INFO field will be used.

The usage of the `generate config` command is as follows.

```
USAGE:
    vcf2rdf generate config [OPTIONS] <FILE>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -a, --assembly <assembly>    Pre-defined assembly [possible values: GRCh37, GRCh38, GRCm38, GRCm39]

ARGS:
    <FILE>    Path to file to process
```


To convert VCF:

```shell
$ vcf2rdf convert --config config.yaml input.vcf.gz > output.ttl
```

The usage of the `convert` command is as follows.

```
USAGE:
    vcf2rdf convert [FLAGS] [OPTIONS] <input> --config <config>

FLAGS:
    -h, --help            Prints help information
        --no-normalize    Do not normalize faldo representation
        --rehearsal       Processes only one record and exit
    -V, --version         Prints version information

OPTIONS:
    -c, --config <config>      Path to configuration yaml
    -s, --subject <subject>    Strategy to generate a subject (use blank node if not specified). If use `id`, ensure
                               that all values at ID column are present and unique [possible values: id, location,
                               reference, normalized_location, normalized_reference]

ARGS:
    <input>    Path to file to process
```

#### Use docker

```shell
$ docker run --rm -v $(pwd):/work togovar/vcf2rdf vcf2rdf generate config --assembly GRCh38 /work/input.vcf.gz > config.yaml
$ docker run --rm -v $(pwd):/work togovar/vcf2rdf vcf2rdf convert --config /work/config.yaml /work/input.vcf.gz > output.ttl
```
