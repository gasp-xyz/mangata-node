#!/bin/bash -e
REPO_ROOT=$(readlink -f $(dirname $(dirname $(readlink -f $0))))

DOCKER_BUILDER_IMAGE=mangatasolutions/node-builder:multi-nightly-2022-11-15 \
    CODE_ROOT=${REPO_ROOT}/mvr \
    ${REPO_ROOT}/docker-cargo.sh \
    build --release

SKIP_BUILD=1 \
    BUILD_DIR=./mvr/docker-cargo/release \
    ${REPO_ROOT}/scripts/build-image.sh mangatasolutions/mangata-node:polkadot-mvr

