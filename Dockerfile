ARG BASE_IMAGE=ekidd/rust-musl-builder:nightly-2020-04-10
FROM ${BASE_IMAGE} AS builder

ADD --chown=rust:rust . .
RUN cargo build --release

FROM alpine:3.11

WORKDIR /app
RUN apk --no-cache add ca-certificates
COPY --from=builder \
    /home/rust/src/target/x86_64-unknown-linux-musl/release/ \
    /usr/local/bin/