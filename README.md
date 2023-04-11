<p align="center">
    <a href="https://https://mangata.finance/">
    <img width="132" height="101" src="https://mangata.finance/images/logo-without-text.svg" class="attachment-full size-full" alt="Mangata brand" loading="lazy" /></a>
</p>

<h2 align="center">Mangata Node</h2>

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

## Build mangata-node locally
- Install [docker](https://docs.docker.com/engine/install/ubuntu/) 

### Compilie mangata-node binary and wasms artifacts
- use docker wrapper for cargo to build `mangata-node`

```
./docker-cargo.sh build --release -p mangata-node
```

build artifacts will be placed in `<REPO ROOT>/docker-cargo/release`

### Generate docker image
You can use `build-image.sh` script to build & generate docker image

```
./scripts/build-image.sh mangatasolutions/mangata-node:dev
```

or you can use already compiled build atributes generated in previous step

```
SKIP_BUILD=1 BUILD_DIR=./docker-cargo/release ./scripts/build-image.sh
```

This will generate new local docker image `mangatasolutions/mangata-node:dev`

## Run

In order to run mangata-parachain locally one need to set up both:
- local relay network
- local parachain network

Because of number of parameters is quite troublesome thats why we came up with dedicated dockerized environment.

### Set up network using parachain-launch

Dockerized setup requires you to build development docker image [mangatasolutions/mangata-node:dev](#generate-docker-image).

Start docker environment using, you need to 

```bash
cd ./launch
yarn install
yarn gen
yarn up
```

once started, you can access nodes using port forwards
- [127.0.0.1:9944](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9946#/explorer) - relaychain 1st collator
- [127.0.0.1:9945](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9946#/explorer) - relaychain 2nd collator
- [127.0.0.1:9946](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9946#/explorer) - parachain 1st collator
- [127.0.0.1:9947](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9946#/explorer) - parachain 2nd collator

Docker setup can be stopped using 

```bash
cd ./launch
yarn down
```

### Sudo access
`Alice` is set as sudo account for parachain-launch docker setup

## Managata node configuration

There is number of chain configurations available for both development and production environements:

| chainspec (`--chain`)         |      Sudo      |  Description                     |
|-------------------------------|----------------|----------------------------------|
| `kusama`                      |    *******     | production kusama public mainnet |
| `kusama-local`                |     Alice      | development kusama local testnet |
| `rococo`                      |    *******     | production rococo public testnet |
| `rococo-local`                |     Alice      | development rococo local testnet |

