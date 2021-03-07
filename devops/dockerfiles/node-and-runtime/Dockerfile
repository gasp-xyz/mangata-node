FROM liuchong/rustup:stable AS builder
LABEL description="Compiles all workspace artifacts"
WORKDIR /mangata
COPY . /mangata

ENV ETH_APP_ID=0xdd514baa317bf095ddba2c0a847765feb389c6a0
ENV ERC20_APP_ID=0x00e392c04743359e39f00cd268a5390d27ef6b44

RUN rustup toolchain install nightly-2020-10-01
RUN rustup target add wasm32-unknown-unknown --toolchain nightly-2020-10-01
RUN WASM_BUILD_TOOLCHAIN=nightly-2020-10-01 cargo build --release

FROM debian:stretch
LABEL description="mangata node"
WORKDIR /mangata
COPY --from=builder /mangata/target/release/mangata-node /mangata/node
# COPY --from=builder /mangata/target/release/wbuild/mangata-runtime/mangata_runtime.compact.wasm /mangata/runtime.compact.wasm

RUN /mangata/node --version

# RUN b2sum -l 256 /mangata/runtime.compact.wasm
# RUN b2sum -l 512 /mangata/runtime.compact.wasm

# EXPOSE 30333 9933 9944

# VOLUME ["/data", "/keystore"]

ENTRYPOINT ["/mangata/node"]
