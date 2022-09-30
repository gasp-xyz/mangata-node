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
ENV SKIP_WASM_BUILD=true
COPY . .
RUN cargo build --release -p mangata-node --no-default-features --features=mangata-rococo,mangata-kusama \
  && mv /app/target/release/mangata-node/mangata-node /app/node \ 
  && rm -rf /app/target /usr/local/cargo /root/.cache

FROM debian:buster
RUN mkdir /mangata
COPY --from=builder /app/node  /mangata/node
ENTRYPOINT ["/mangata/node"]