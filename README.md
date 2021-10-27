# Mangata node
## AMM DEX blockchain without fee market, that incorporates MEV prevention
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

Recommended rustc version for the build is `nightly-2020-10-01`

Environment variables for ethereum apps should be set up before the build:
```bash
ETH_APP_ID=0xdd514baa317bf095ddba2c0a847765feb389c6a0
ENV ERC20_APP_ID=0x00e392c04743359e39f00cd268a5390d27ef6b44
```
build node:
```bash
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


