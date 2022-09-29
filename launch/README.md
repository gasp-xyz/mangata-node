## Description
Provides an easy way to set up a local chain based on .yml configs.
For the Mangata nodes we are currently using the dev Docker image, which needs to be built locally.
We rely on the `@open-web3/parachain-launch` tooling, with slight modifications specific to our chain setup.
In case we need to extend the functionality of the tooling, the code resides in `https://github.com/mangata-finance/parachain-launch`.

## Run
build the mangata node & create image
```shell
export SKIP_BUILD=1
./docker-cargo.sh build --release --features=mangata-rococo
scripts/build-image.sh

# cd/open new terminal in mangata-node/launch
cd launch
# install the deps and compile forked 'parachain-launch'
yarn install
# for relay + mangata
yarn gen
# for relay + mangata + karura or other custom
yarn gen-karura
# start nodes
yarn up
# stop nodes
yarn down
```
# Test some change