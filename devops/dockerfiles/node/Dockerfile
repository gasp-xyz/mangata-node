#
# RUST_CONTAINTER - base rust tooling, based on https://github.com/liuchong/docker-rustup
#
FROM debian:buster as rustup
WORKDIR /root

# common packages
RUN apt-get update && \
    apt-get install --no-install-recommends -y \
    ca-certificates curl file \
    build-essential \
    autoconf automake autotools-dev libtool xutils-dev && \
    rm -rf /var/lib/apt/lists/*

ENV SSL_VERSION=1.0.2u

RUN curl https://www.openssl.org/source/openssl-$SSL_VERSION.tar.gz -O && \
    tar -xzf openssl-$SSL_VERSION.tar.gz && \
    cd openssl-$SSL_VERSION && ./config && make depend && make install && \
    cd .. && rm -rf openssl-$SSL_VERSION*

ENV OPENSSL_LIB_DIR=/usr/local/ssl/lib \
    OPENSSL_INCLUDE_DIR=/usr/local/ssl/include \
    OPENSSL_STATIC=1

# install toolchain
RUN curl https://sh.rustup.rs -sSf | \
    sh -s -- --default-toolchain stable -y

ENV PATH=/root/.cargo/bin:$PATH

#
# BUILDER - mangata tooling required for building mangata-node
#
FROM rustup AS builder
LABEL description="Compiles all workspace artifacts"
#Export memory Profile env. variables.
ENV MEMORY_PROFILER_OUTPUT=profiling_%e_%t.dat 
ENV MEMORY_PROFILER_CULL_TEMPORARY_ALLOCATIONS=1
ENV MEMORY_PROFILER_LOG=info
ENV MEMORY_PROFILER_LOGFILE=profiling_%e_%t.log 
ENV ETH_APP_ID=0xdd514baa317bf095ddba2c0a847765feb389c6a0
ENV ERC20_APP_ID=0x00e392c04743359e39f00cd268a5390d27ef6b44

RUN apt-get update && apt-get install -y cmake pkg-config libssl-dev git build-essential clang libclang-dev curl llvm nodejs npm
RUN npm install --global yarn

RUN rustup install nightly-2021-10-19
RUN rustup default nightly-2021-10-19
RUN rustup target add wasm32-unknown-unknown --toolchain nightly-2021-10-19
RUN cargo install cargo2junit

RUN git clone --depth 1 --branch 0.8.0 https://github.com/koute/memory-profiler /tmp/profiler
RUN cargo build --manifest-path /tmp/profiler/Cargo.toml --release -p bytehound-preload -p bytehound-cli

RUN chmod 755 /root/
RUN chmod 755 -R /root/.cargo/
RUN chmod 755 -R /root/.rustup/


FROM debian:buster as runner_core
COPY --from=builder /tmp/profiler/target/release/bytehound /usr/bin/bytehound
COPY --from=builder /tmp/profiler/target/release/libbytehound.so /usr/lib/libbytehound.so
ENV DOCKERIZE_VERSION v0.6.1
ENV VAULT_VERSION 1.9.3
RUN apt-get update && apt-get install -y curl netcat jq unzip
RUN curl -L https://github.com/jwilder/dockerize/releases/download/${DOCKERIZE_VERSION}/dockerize-alpine-linux-amd64-${DOCKERIZE_VERSION}.tar.gz --output /tmp/dockerize.tar.gz
RUN curl -L https://releases.hashicorp.com/vault/${VAULT_VERSION}/vault_${VAULT_VERSION}_linux_amd64.zip --output /tmp/vault.zip
RUN tar -C /usr/bin -xzvf /tmp/dockerize.tar.gz
RUN unzip /tmp/vault.zip -d /usr/bin
COPY devops/dockerfiles/node/unlocker.sh /unlocker.sh
COPY devops/dockerfiles/node/entrypoint.sh /entrypoint.sh
RUN mkdir /mangata

FROM runner_core as runner
ARG NODE_BINARY
ARG WASM
COPY $NODE_BINARY /mangata/node
COPY $WASM /mangata/
ENTRYPOINT /entrypoint.sh
