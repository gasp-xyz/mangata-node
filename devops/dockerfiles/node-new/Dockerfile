# syntax=docker/dockerfile:1
FROM paritytech/ci-linux:production AS base
RUN cargo install cargo-chef
WORKDIR /app

FROM base AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json --bin node

FROM base AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN --mount=type=cache,target=/app/target \
  cargo +nightly chef cook --release --recipe-path recipe.json --no-default-features --bin mangata-node

# Build application
COPY . .
RUN --mount=type=cache,target=/app/target \
  cargo +nightly build --release --features=mangata-rococo \
  && mv /app/target/release/mangata-node /app/node \
  && mv /app/target/release/wbuild/mangata-rococo-runtime/mangata_rococo_runtime.compact.compressed.wasm /app/

FROM debian:buster
RUN mkdir /mangata
COPY --from=builder /app/node  /mangata/node
COPY --from=builder /app/mangata_rococo_runtime.compact.compressed.wasm  /mangata/
ENTRYPOINT ["/mangata/node"]