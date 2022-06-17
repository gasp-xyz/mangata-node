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

DOCKER_IMAGE_TAG=${@:-mangatasolutions/mangata-node:dev}
DOCKER_TAGS=""

echo "building docker image ${DOCKER_IMAGE_TAG} with tags:"
for tag in ${DOCKER_IMAGE_TAG}; do
    echo \t - ${tag}
    DOCKER_TAGS="${DOCKER_TAGS} -t ${tag}"
done

if [ ! -e ${NODE_BINARY} ]; then
    echo "env variable 'NODE_BINARY' : ${NODE_BINARY} not found" >&2
    exit -1
fi

if [ ! -e ${WASM} ]; then
    echo "env variable 'WASM' : ${WASM} not found" >&2
    exit -1
fi


docker build \
    --build-arg WASM=${WASM} \
    --build-arg NODE_BINARY=${NODE_BINARY} \
    --label "git_rev=${DOCKER_LABEL}" \
    ${DOCKER_TAGS} \
    -f ${REPO_ROOT}/devops/dockerfiles/node/Dockerfile \
    ${REPO_ROOT}
