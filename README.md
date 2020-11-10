# Mangata node
## The fair DEX
The most reliable decentralized exchange (DEX) - capable of working across multiple blockchains using interoperability. The exchange is using an innovative consensus algorithm that solves frontrunning problems and makes all participants equal. 

Our mission is to bring fair and equal rules for exchange participants, fix volatile fees of trading and reuse exchange assets to maximize gains. Our goal is to create the number one Go-To protocol for traders and liquidity providers of digital currencies online.

The design of our blockchain guarantees low fixed-fees that provides greater predictability of trading costs.
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

```bash
cargo build --release
```

### Docker container

```bash
./scripts/build-manga-node-docker-image.sh
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
