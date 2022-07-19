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
ENV RUST_TOOLCHAIN=nightly-2022-05-11

ENV RUSTUP_HOME=/opt/rust
ENV CARGO_HOME=/opt/cargo
ENV PATH=/opt/rust/bin:$PATH
ENV PATH=/opt/cargo/bin:$PATH
RUN curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain ${RUST_TOOLCHAIN} -y --no-modify-path
RUN rustup target add wasm32-unknown-unknown --toolchain ${RUST_TOOLCHAIN}

#
# BUILDER - mangata tooling required for building mangata-node
#
FROM rustup AS builder_core
LABEL description="Compiles all workspace artifacts"
#Export memory Profile env. variables.
ENV MEMORY_PROFILER_OUTPUT=profiling_%e_%t.dat
ENV MEMORY_PROFILER_CULL_TEMPORARY_ALLOCATIONS=1
ENV MEMORY_PROFILER_LOG=info
ENV MEMORY_PROFILER_LOGFILE=profiling_%e_%t.log
ENV ETH_APP_ID=0xdd514baa317bf095ddba2c0a847765feb389c6a0
ENV ERC20_APP_ID=0x00e392c04743359e39f00cd268a5390d27ef6b44

RUN apt-get update \
	&& apt-get install -y \
	cmake \
	pkg-config \
	libssl-dev \
	git \
	build-essential \
	clang \
	libclang-dev \
	curl \
	llvm \
	nodejs \
	npm \
	&& rm -rf /var/lib/apt/lists/*

RUN apt-get update && apt-get -y install cmake protobuf-compiler

RUN npm install --global yarn

RUN cargo install cargo2junit \
	&& rm -rf ${CARGO_HOME}/git \
	&& rm -rf ${CARGO_HOME}/registry

RUN chmod 777 /opt/
RUN chmod 777 /opt/cargo/
