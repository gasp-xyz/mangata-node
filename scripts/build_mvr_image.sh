#!/bin/bash
REPO_ROOT=$(readlink -f $(dirname $(dirname $(readlink -f $0))))

CODE_ROOT=/home/dev/mangata-node/mvr \
    SKIP_BUILD=1 \
    BUILD_DIR=${REPO_ROOT}/mvr/docker-cargo/release \
    ${REPO_ROOT}/scripts/build_mvr_image.sh

