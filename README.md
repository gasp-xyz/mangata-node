<p align="center">
    <a href="https://https://mangata.finance/">
    <img width="132" height="101" src="https://mangata.finance/images/logo-without-text.svg" class="attachment-full size-full" alt="Mangata brand" loading="lazy" /></a>
</p>


<h2 align="center">Mangata Node</h2>

<p align="center">
    Omnichain zk-rollup for L1-grade native liquidity. Implementation includes <a href="https://blog.mangata.finance/blog/2021-10-10-themis-protocol/" target="_blank" rel="noopener noreferrer">MEV solution</a>, Proof-of-Liquidity, gas-free swaps, algorithmic buy & burn, weight voting & liquidity gauges, time-incentivized liquidity provision, 3rd party incentives, and more.
</p>

![Themis](https://blog.mangata.finance/assets/posts/themis-cover.png)

![Issues](https://img.shields.io/github/issues/mangata-finance/mangata-node)
![Pull Request](https://img.shields.io/github/issues-pr/mangata-finance/mangata-node)
![GitHub last commit](https://img.shields.io/github/last-commit/mangata-finance/mangata-node)
![Build Status](https://img.shields.io/endpoint.svg?url=https%3A%2F%2Factions-badge.atrox.dev%2Fmangata-finance%2Fmangata-node%2Fbadge%3Fref%3Ddevelop&style=flat)
![Language](https://img.shields.io/github/languages/top/mangata-finance/mangata-node)

## Description

Mangata operates as a cross-chain liquidity protocol, facilitating seamless transactions between Ethereum and various other blockchains through a omnichain zk-rollup Infrastructure. We leverage the power of ZK-rollup, a second-layer (L2) solution, to ensure universal connectivity with first-layer (L1) blockchains. Additionally, our decentralized exchange platform is designed to provide robust protection against Miner Extractable Value (MEV) and frontrunning attempts, thereby safeguarding the interests of our users.

## API

[Mangata API Docs](https://mangata-finance.notion.site/Mangata-API-Docs-06f68bc6ba004416ae5c6686163b0468)

## Build rollup-node locally
- Install [docker](https://docs.docker.com/engine/install/ubuntu/)

### Compile rollup-node binary and wasms artifacts
- use docker wrapper for cargo to build `rollup-node`

```
./docker-cargo.sh build --release -p rollup-node
```

build artifacts will be placed in `<REPO ROOT>/docker-cargo/release`

### Run tests and generate code coverage report
Run unit tests only:
```bash
cargo test
```
Run unit tests and generate code coverage report in html format:
```bash
cargo install cargo-tarpaulin
cargo tarpaulin --timeout 120 --workspace -e rollup-runtime-integration-test rollup-node --exclude-files **/mock.rs **/weights.rs **/weights/* --out Html
```

### Generate docker image
You can use `build-image.sh` script to build & generate docker image

```
./scripts/build-image.sh mangatasolutions/rollup-node:dev
```

or you can use already compiled build atributes generated in previous step

```
SKIP_BUILD=1 BUILD_DIR=./docker-cargo/release ./scripts/build-image.sh
```

This will generate new local docker image `mangatasolutions/rollup-node:dev`

## Run

In order to run the rollup-node locally one need to set up:
- local rollup network

Because of number of parameters is quite troublesome thats why we came up with dedicated dockerized environment.

### Set up network using docker-compose

Dockerized setup requires you to build development docker image [mangatasolutions/rollup-node:dev](#generate-docker-image).

Start docker environment using docker-compose, you need to

```bash
cd ./devops/launch
docker compose up
```

once started, you can access nodes using port forwards
- [127.0.0.1:9944](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/explorer) - rollup 1st node
- [127.0.0.1:9945](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9945#/explorer) - rollup 2nd node

Docker setup can be stopped using

```bash
cd ./devops/launch
docker compose down
```

### Sudo access
`Alice` is set as sudo account for docker setup

## Mangata node configuration

There is number of chain configurations available for both development and production environements:

| chainspec (`--chain`)         |      Sudo      |                           Description                         |
|-------------------------------|----------------|---------------------------------------------------------------|
| `rollup-local`                |    *******     | development rollup local testnet                              |
| `rollup-local-seq`            |     Alice      | development rollup local testnet with collators as sequencers |

