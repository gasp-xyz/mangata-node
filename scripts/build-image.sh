#!/bin/bash
REPO_ROOT=$(readlink -f $(dirname $(dirname $(readlink -f $0))))
${REPO_ROOT}/docker-cargo.sh build --release
BUILD_DIR=${REPO_ROOT}/docker-build/release
NODE_BINARY=${BUILD_DIR}/mangata-node
WASM=${BUILD_DIR}/wbuild/mangata-runtime/mangata_runtime.compact.compressed.wasm
GIT_REV=$(git -C ${REPO_ROOT} rev-parse HEAD)

if git -C ${REPO_ROOT} diff --quiet HEAD; then
    DOCKER_LABEL=${GIT_REV}
else
    DOCKER_LABEL=${GIT_REV}-dirty
fi

DOCKER_IMAGE_TAG=${1:-local}

if [ ! -e ${NODE_BINARY} ]; then
    echo "${NODE_BINARY} not found" >&2
    exit -1
fi

if [ ! -e ${WASM} ]; then
    echo "${WASM} not found" >&2
    exit -1
fi

docker build \
    -f ${REPO_ROOT}/devops/dockerfiles/node/Dockerfile
    -arg WASM=${WASM} \
    -arg NODE_BINARY=${NODE_BINARY} \
    --label "git_rev=${DOCKER_LABEL}"
    -t mangatasolutions:mangata-node:${DOCKER_IMAGE_TAG} \
    ${REPO_ROOT}
