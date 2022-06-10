#!/bin/bash -xe
REPO_ROOT=$(readlink -f $(dirname $(dirname $(readlink -f $0))))
if [ -z "${SKIP_BUILD}" ]; then 
    ${REPO_ROOT}/docker-cargo.sh build --release
else
    echo "build skipped because SKIP_BUILD flag is set"
fi
BUILD_DIR=docker-cargo/release
NODE_BINARY=${NODE_BINARY:-${BUILD_DIR}/mangata-node}
WASM=${WASM:-${BUILD_DIR}/wbuild/mangata-rococo-runtime/mangata_rococo_runtime.compact.compressed.wasm}
GIT_REV=$(git -C ${REPO_ROOT} rev-parse HEAD)

if git -C ${REPO_ROOT} diff --quiet HEAD; then
    DOCKER_LABEL=${GIT_REV}
else
    DOCKER_LABEL=${GIT_REV}-dirty
fi

DOCKER_IMAGE_TAG=${1:-mangatasolutions/mangata-node:local}

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
