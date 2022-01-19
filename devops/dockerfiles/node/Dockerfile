FROM liuchong/rustup:stable AS env_builder
LABEL description="Compiles all workspace artifacts"
WORKDIR /mangata
COPY . /mangata

ENV ETH_APP_ID=0xdd514baa317bf095ddba2c0a847765feb389c6a0
ENV ERC20_APP_ID=0x00e392c04743359e39f00cd268a5390d27ef6b44

RUN apt-get update && apt-get install -y cmake pkg-config libssl-dev git build-essential clang libclang-dev curl llvm

RUN rustup install nightly-2021-10-19
RUN rustup default nightly-2021-10-19
RUN rustup target add wasm32-unknown-unknown --toolchain nightly-2021-10-19
RUN cargo install cargo2junit

FROM env_builder as builder
RUN cargo build --release

FROM debian:stretch
WORKDIR /mangata
RUN apt-get update && apt-get install -y curl
COPY --from=builder /mangata/target/release/mangata-node /mangata/node
RUN /mangata/node --version
COPY devops/dockerfiles/node/runner.sh /runner.sh

ENTRYPOINT ["/runner.sh"]
