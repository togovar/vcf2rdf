FROM rust:alpine

RUN apk add --no-cache musl-dev

COPY . /tmp/vcf2rdf

WORKDIR /tmp/vcf2rdf

RUN cargo build --release

FROM alpine:3.14

COPY --from=0 /tmp/vcf2rdf/target/release/vcf2rdf /usr/local/bin/
