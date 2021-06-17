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
    convert     Converts VCF to RDF
    generate    Generates template
    help        Prints this message or the help of the given subcommand(s)
    stat        Prints statistics
```

### Convert VCF to RDF

```
USAGE:
    vcf2rdf convert [FLAGS] [OPTIONS] <input> --assembly <assembly>

FLAGS:
    -h, --help         Prints help information
        --rehearsal    Processes only one record and exit
    -V, --version      Prints version information

OPTIONS:
    -a, --assembly <assembly>        Assembly. (e.g. GRCh37, GRCh38)
    -c, --config <config>            Path to configuration yaml
    -s, --subject-id <subject-id>    Strategy to generate subject ID (use blank node if not specified). If use `id`,
                                     ensure that all values at ID column are present and unique [possible values: id,
                                     location]

ARGS:
    <input>    Path to file to process
```

To generate config template:

```
$ vcf2rdf generate config <input.vcf>
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
```

All keys listed in VCF meta-information lines (##INFO=<...>) are filled in info.
Remove key names that you do not need in converted RDF.
Without a configuration file, all values in the INFO field will be used.

#### with docker

```shell
$ docker run --rm -v $(pwd):/work togovar/vcf2rdf vcf2rdf convert -a GRCh37 /work/input.vcf.gz
```
