## Description
Provides an easy way to set up a local chain based on .yml configs.
For the Mangata nodes we are currently using the dev Docker image, which needs to be built locally.

## Run
build the mangata node & create image
```shell
export SKIP_BUILD=1
./docker-cargo.sh build --release --features=enable-trading,mangata-rococo
scripts/build-image.sh

# cd/open terminal in mangata-node/launch
cd launch
# for relay + mangata
yarn gen
# for relay + mangata + karura or other custom
yarn gen-karura
# start nodes
yarn up
# stop nodes
yarn down
```
