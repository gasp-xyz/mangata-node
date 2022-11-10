# syntax=docker/dockerfile:1
# Referenced digest is for `paritytech/ci-linux:production` image tag used for immutability
FROM paritytech/ci-linux@sha256:535c8a792a618f549832fa57e52c346d352021fa050ec82b30073810eb15369e AS base
RUN cargo install cargo-chef
WORKDIR /app

FROM base AS planner
COPY . .
RUN cargo +nightly chef prepare

FROM base AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo +nightly chef cook --check --release --no-default-features -p mangata-node

# Build application
COPY . .
RUN cargo build --release -p mangata-node --no-default-features --features=mangata-rococo,mangata-kusama,runtime-benchmarks \
  && mv /app/target/release/mangata-node /app/mangata-node \
  && mv /app/target/release/wbuild/mangata-rococo-runtime/mangata_rococo_runtime.compact.compressed.wasm /app/ \
  && mv /app/target/release/wbuild/mangata-kusama-runtime/mangata_kusama_runtime.compact.compressed.wasm /app/ \
  && rm -rf /app/target /usr/local/cargo /root/.cache 

FROM debian:buster
WORKDIR /mangata

COPY --from=builder /app/mangata-node  /mangata/node
COPY --from=builder /app/templates  /mangata/templates
COPY --from=builder /app/mangata_rococo_runtime.compact.compressed.wasm  /mangata/
COPY --from=builder /app/mangata_kusama_runtime.compact.compressed.wasm  /mangata/
ENTRYPOINT ["/mangata/node"]
