#!/bin/bash -e
REPO_ROOT=$(readlink -f $(dirname $(dirname $(readlink -f $0))))
${REPO_ROOT}/docker-cargo.sh build --release
BUILD_DIR=docker-build/release
NODE_BINARY=${BUILD_DIR}/mangata-node
WASM=${BUILD_DIR}/wbuild/mangata-runtime/mangata_runtime.compact.compressed.wasm
GIT_REV=$(git -C ${REPO_ROOT} rev-parse HEAD)

if git -C ${REPO_ROOT} diff --quiet HEAD; then
    DOCKER_LABEL=${GIT_REV}
else
    DOCKER_LABEL=${GIT_REV}-dirty
fi

DOCKER_IMAGE_TAG=mangatasolutions/mangata-node:${1:-local}

if [ ! -e ${NODE_BINARY} ]; then
    echo "${NODE_BINARY} not found" >&2
    exit -1
fi

if [ ! -e ${WASM} ]; then
    echo "${WASM} not found" >&2
    exit -1
fi

echo "building docker image ${DOCKER_IMAGE_TAG}"

docker build \
    --build-arg WASM=${WASM} \
    --build-arg NODE_BINARY=${NODE_BINARY} \
    --label "git_rev=${DOCKER_LABEL}" \
    -t ${DOCKER_IMAGE_TAG} \
    -f ${REPO_ROOT}/devops/dockerfiles/node/Dockerfile \
    ${REPO_ROOT}
