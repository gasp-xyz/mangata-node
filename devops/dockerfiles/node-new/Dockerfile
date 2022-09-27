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
RUN --mount=type=cache,target=/usr/local/cargo/git \
  --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/root/.cache/sccache \
  --mount=type=cache,target=/app/target/release \
  cargo +nightly chef cook --release --recipe-path recipe.json --no-default-features --bin mangata-node

# # Build application
# COPY . .
# RUN cargo build --release