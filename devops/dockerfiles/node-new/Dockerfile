# syntax=docker/dockerfile:1
FROM paritytech/ci-linux:production AS base
RUN cargo install cargo-chef
WORKDIR /app

FROM base AS planner
COPY . .
RUN cargo chef prepare

FROM base AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --check --release --no-default-features -p mangata-node

# Build application
COPY . .
RUN cargo build --release -p mangata-node --no-default-features --features=mangata-rococo,mangata-kusama \
  && mv /app/target/release/mangata-node /app/node \ 
  && mv /app/target/release/wbuild/mangata-rococo-runtime/mangata_rococo_runtime.compact.compressed.wasm /app/ \
  && mv /app/target/release/wbuild/mangata-kusama-runtime/mangata_kusama_runtime.compact.compressed.wasm /app/ \
  && rm -rf /app/target /usr/local/cargo /root/.cache 

FROM debian:buster
RUN mkdir /mangata
COPY --from=builder /app/node  /mangata/node
COPY --from=builder /app/mangata_rococo_runtime.compact.compressed.wasm  /mangata/
COPY --from=builder /app/mangata_kusama_runtime.compact.compressed.wasm  /mangata/
ENTRYPOINT ["/mangata/node/mangata-node"]