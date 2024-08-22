FROM rust:1.71-buster AS builder

# install tools and dependencies
RUN set -eux && \
		apt-get -y update; \
		apt-get install -y --no-install-recommends \
		libssl-dev make cmake graphviz clang libclang-dev llvm git pkg-config curl time rhash ca-certificates \
		python3 python3-pip lsof ruby ruby-bundler git-restore-mtime xz-utils unzip gnupg protobuf-compiler && \
		# apt clean up
		apt-get autoremove -y && apt-get clean && rm -rf /var/lib/apt/lists/*

ARG RUST_TOOLCHAIN=nightly-2023-05-22

RUN rustup install $RUST_TOOLCHAIN && rustup default $RUST_TOOLCHAIN && \
	rustup target add wasm32-unknown-unknown && \
	rustup component add rust-src rustfmt clippy && \
	# cargo install --git https://github.com/paritytech/try-runtime-cli --tag v0.3.3 --locked && \
	# removes compilation artifacts cargo install creates (>250M)
	rm -rf "${CARGO_HOME}/registry" "${CARGO_HOME}/git"
	
# show backtraces
ENV	RUST_BACKTRACE=1

WORKDIR /app

COPY . .

ARG ENABLE_FAST_RUNTIME=false

RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/app/target,sharing=locked \
    if [ "$ENABLE_FAST_RUNTIME" = "true" ]; then \
        cargo build --release --no-default-features --features=fast-runtime; \
    else \
        cargo build --locked --release; \
    fi \
		# copy build artifacts to the root directory to avoid issues with accessing cache mount from 2nd stage
		&& cp target/release/rollup-node ./node \
		&& cp target/release/wbuild/rollup-runtime/rollup_runtime.compact.compressed.wasm ./rollup_runtime.compact.compressed.wasm 

FROM debian:buster AS runner

WORKDIR /app

COPY --from=builder /app/node /app/rollup_runtime.compact.compressed.wasm /app/

ENTRYPOINT ["/app/node"]
