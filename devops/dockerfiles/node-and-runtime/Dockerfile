FROM mangata/rust-builder AS builder
LABEL description="Compiles all workspace artifacts"
WORKDIR /mangata
COPY . /mangata

RUN WASM_BUILD_TOOLCHAIN=nightly-2020-05-23 cargo build --release

FROM debian:stretch
LABEL description="mangata node"
WORKDIR /mangata
COPY --from=builder /mangata/target/release/node-template /mangata/node
COPY --from=builder /mangata/target/release/wbuild/node-template-runtime/node_template_runtime.compact.wasm /mangata/runtime.compact.wasm

RUN /mangata/node --version

RUN b2sum -l 256 /mangata/runtime.compact.wasm
RUN b2sum -l 512 /mangata/runtime.compact.wasm

EXPOSE 30333 9933 9944

VOLUME ["/data", "/keystore"]

ENTRYPOINT ["/mangata/node"]
