FROM liuchong/rustup:stable AS env_builder
LABEL description="Compiles all workspace artifacts"
WORKDIR /mangata
COPY . /mangata

#Export memory Profile env. variables.
ENV MEMORY_PROFILER_OUTPUT=profiling_%e_%t.dat 
ENV MEMORY_PROFILER_CULL_TEMPORARY_ALLOCATIONS=1
ENV MEMORY_PROFILER_LOG=info
ENV MEMORY_PROFILER_LOGFILE=profiling_%e_%t.log 
ARG CARGO_FEATURES=""


ENV ETH_APP_ID=0xdd514baa317bf095ddba2c0a847765feb389c6a0
ENV ERC20_APP_ID=0x00e392c04743359e39f00cd268a5390d27ef6b44

RUN apt-get update && apt-get install -y cmake pkg-config libssl-dev git build-essential clang libclang-dev curl llvm nodejs npm
RUN npm install --global yarn

RUN rustup install nightly-2021-10-19
RUN rustup default nightly-2021-10-19
RUN rustup target add wasm32-unknown-unknown --toolchain nightly-2021-10-19
#Get Mem profiler binaries

RUN cargo install cargo2junit

FROM env_builder as builder
RUN mkdir memory_profiler && cd memory_profiler && git clone https://github.com/koute/memory-profiler && cd memory-profiler && git checkout  0.8.0  &&  cargo build --release -p bytehound-preload && cargo build --release -p bytehound-cli
RUN cp /mangata/memory_profiler/memory-profiler/target/release/libbytehound.so /mangata/
RUN cp /mangata/memory_profiler/memory-profiler/target/release/bytehound /mangata/


RUN cargo build --release --features=${CARGO_FEATURES}

FROM debian:stretch
ENV DOCKERIZE_VERSION v0.6.1
ENV VAULT_VERSION 1.9.3
RUN apt-get update && apt-get install -y curl netcat jq unzip
RUN curl -L https://github.com/jwilder/dockerize/releases/download/${DOCKERIZE_VERSION}/dockerize-alpine-linux-amd64-${DOCKERIZE_VERSION}.tar.gz --output /tmp/dockerize.tar.gz
RUN curl -L https://releases.hashicorp.com/vault/${VAULT_VERSION}/vault_${VAULT_VERSION}_linux_amd64.zip --output /tmp/vault.zip
RUN tar -C /usr/bin -xzvf /tmp/dockerize.tar.gz
RUN unzip /tmp/vault.zip -d /usr/bin
WORKDIR /mangata
COPY --from=builder /mangata/target/release/mangata-node /mangata/node
COPY --from=builder /mangata/target/release/wbuild/mangata-rococo-runtime/mangata_rococo_runtime.compact.compressed.wasm /mangata/mangata_rococo_runtime.compact.compressed.wasm
COPY --from=builder /mangata/target/release/wbuild/mangata-kusama-runtime/mangata_kusama_runtime.compact.compressed.wasm /mangata/mangata_kusama_runtime.compact.compressed.wasm
COPY --from=builder /mangata/libbytehound.so /mangata/libbytehound.so
COPY --from=builder /mangata/bytehound /mangata/bytehound
RUN /mangata/node --version
COPY devops/dockerfiles/node/unlocker.sh /unlocker.sh
COPY devops/dockerfiles/node/entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh
RUN chmod +x /unlocker.sh

ENTRYPOINT ["/entrypoint.sh"]
