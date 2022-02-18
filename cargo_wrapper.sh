#!/bin/bash
export WASM_TARGET_DIRECTORY="$(pwd)"
REPO_ROOT=$(dirname $(readlink -f $0))
docker build --target env_builder  -f ./devops/dockerfiles/node/Dockerfile -t mangatasolutions/cargo-wrapper .
docker run --rm -e CARGO_TARGET_DIR=/mangata/output -v $REPO_ROOT:/mangata  -it mangatasolutions/cargo-wrapper /bin/bash -c "\
/root/.cargo/bin/cargo $@; \
chown -R $UID:$UID /mangata/output"
