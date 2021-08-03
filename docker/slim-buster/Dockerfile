FROM rust:buster

COPY . /tmp/vcf2rdf

WORKDIR /tmp/vcf2rdf

RUN cargo build --release

FROM debian:buster-slim

COPY --from=0 /tmp/vcf2rdf/target/release/vcf2rdf /usr/local/bin/
