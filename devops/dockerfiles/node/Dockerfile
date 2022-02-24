FROM liuchong/rustup:stable AS env_builder
LABEL description="Compiles all workspace artifacts"
WORKDIR /mangata
COPY . /mangata

#Export memory Profile env. variables.
RUN export MEMORY_PROFILER_LOG=info && export MEMORY_PROFILER_LOGFILE=profiling_%e_%t.log && export MEMORY_PROFILER_OUTPUT=profiling_%e_%t.dat && export MEMORY_PROFILER_CULL_TEMPORARY_ALLOCATIONS=1
#Get Mem profiler binaries
RUN curl -L https://github.com/koute/memory-profiler/releases/download/0.6.1/memory-profiler-x86_64-unknown-linux-gnu.tgz -o memory-profiler-x86_64-unknown-linux-gnu.tgz
RUN tar -xf memory-profiler-x86_64-unknown-linux-gnu.tgz
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
COPY --from=builder /mangata/libmemory_profiler.so /mangata/libmemory_profiler.so
COPY --from=builder /mangata/memory-profiler-cli /mangata/memory-profiler-cli
RUN /mangata/node --version
COPY devops/dockerfiles/node/runner.sh /runner.sh

ENTRYPOINT ["LD_PRELOAD=/mangata/libmemory_profiler.so /mangata/node"]
