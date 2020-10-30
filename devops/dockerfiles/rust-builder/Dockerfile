FROM liuchong/rustup:1.46.0 AS builder
LABEL description="Rust and WASM build environment for mangata and substrate"

WORKDIR /setup
COPY setup.sh /setup
ENV TERM=xterm

RUN ./setup.sh
