FROM mangatasolutions/node-builder:multi-nightly-2023-05-22 AS builder

WORKDIR /app

COPY . .

ARG ENABLE_FAST_RUNTIME=false

# Set RUSTC_WRAPPER to empty string by default, as it's causing random issues on arm64 image builds in CI
# To enable sccache set RUSTC_WRAPPER build arg to "sccache" in your image build command with `--build-arg RUSTC_WRAPPER=sccache`
ARG RUSTC_WRAPPER=""

RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/app/target,sharing=locked \
    export RUSTC_WRAPPER=${RUSTC_WRAPPER} && \
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
