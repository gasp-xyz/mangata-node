FROM mangatasolutions/node-builder:multi-nightly-2023-05-22 as planner
WORKDIR /app
RUN cargo install --locked cargo-chef@0.1.62
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM mangatasolutions/node-builder:multi-nightly-2023-05-22 as cacher
WORKDIR /app
RUN cargo install --locked cargo-chef@0.1.62
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM mangatasolutions/node-builder:multi-nightly-2023-05-22 as builder
WORKDIR /app
COPY . .
# Copy over the cached dependencies
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
RUN cargo build --release

FROM debian:stable-slim

WORKDIR /app

ARG BINARY_PATH=target/release/rollup-node
ARG WASM_PATH=target/wbuild/rollup-runtime/rollup_runtime.compact.compressed.wasm

COPY --from=builder /app/${BINARY_PATH} /app/

ENTRYPOINT ["/app/rollup-node"]