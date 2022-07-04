<p align="center">
    <a href="https://https://mangata.finance/">
    <img width="132" height="101" src="https://mangata.finance/images/logo-without-text.svg" class="attachment-full size-full" alt="Mangata brand" loading="lazy" /></a>
</p>

<h3 align="center">Mangata Node</h3>

<p align="center">
    Application-specific blockchain for decentralized exchange, a parachain in Polkadot ecosystem. Implementation includes <a href="https://blog.mangata.finance/blog/2021-10-10-themis-protocol/" target="_blank" rel="noopener noreferrer">MEV solution</a>, Proof of Liquidity and no gas economy.
</p>

![Themis](https://blog.mangata.finance/assets/posts/themis-cover.png)

![Issues](https://img.shields.io/github/issues/mangata-finance/mangata-node)
![Pull Request](https://img.shields.io/github/issues-pr/mangata-finance/mangata-node)
![GitHub last commit](https://img.shields.io/github/last-commit/mangata-finance/mangata-node)
![Build Status](https://img.shields.io/endpoint.svg?url=https%3A%2F%2Factions-badge.atrox.dev%2Fmangata-finance%2Fmangata-node%2Fbadge%3Fref%3Ddevelop&style=flat)
![Language](https://img.shields.io/github/languages/top/mangata-finance/mangata-node)

## Description

Reliable decentralized exchange (DEX) blockchain - interoperable with other blockchains using Polkadot. The exchange is using a consensus algorithm that solves MEV/frontrunning problems and makes all participants' access to trading opportunities equal.

The design of the blockchain guarantees fixed-fees that provides greater control of trading costs and higher arbitrage opportunity.
Assets on the exchange will serve multiple purposes- at the first iteration, they are the block producer’s stake and exchange liquidity at the same time, and more comes later.

## Local Development

Follow these steps to prepare a local Substrate development environment :hammer_and_wrench:

### Simple Setup

Install all the required dependencies with a single command (be patient, this can take up to 30
minutes).

```bash
curl https://getsubstrate.io -sSf | bash -s -- --fast
```

### Manual Setup

Find manual setup instructions at the
[Substrate Developer Hub](https://substrate.dev/docs/en/knowledgebase/getting-started/#manual-installation).

## Build

### Local target

Recommended rustc version for the build is `nightly-2021-10-19`

Environment variables for ethereum apps should be set up before the build:

```bash
ETH_APP_ID=0xdd514baa317bf095ddba2c0a847765feb389c6a0
ERC20_APP_ID=0x00e392c04743359e39f00cd268a5390d27ef6b44
```

build node:

```bash
rustup target add wasm32-unknown-unknown
cargo build --release
```

### Docker container

```bash
./scripts/build-mangata-node-docker-image.sh
```

## Run

### Single Node Development Chain

Purge any existing dev chain state:

```bash
./target/release/mangata-node purge-chain --dev
```

Start a dev chain:

```bash
./target/release/mangata-node --dev
```

### Two-Nodes Dockerized Testnet

```bash
cd ./devops
docker-compose up
```

### Two-Nodes Multi-Validator Dockerized Testnet

```bash
sh scripts/build-multi-validator-mangata-node-docker-image.sh
docker-compose  -f devops/multi-validator-docker-compose.yml up
```

## Debug Single Node

### VS code

Export RUSTFLAGS

```bash
export RUSTFLAGS="-g"
```

Build node:

```bash
cargo build --release
```

Run node:

```bash
RUSTFLAGS="-g" cargo run -j12 --release -- --tmp --dev
```

Go to VS code and attach the process!

# Mangata Substrate Cumulus Parachain
