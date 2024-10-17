#!/bin/bash -xe
REPO_ROOT=$(readlink -f $(dirname $(dirname $(readlink -f $0))))

if [ -z "${SKIP_BUILD}" ]; then
    ${REPO_ROOT}/docker-cargo.sh build --release
else
    echo "build skipped because SKIP_BUILD flag is set"
fi
BINARY_PATH=${BINARY_PATH:-./docker-cargo/release/rollup-node}
GIT_REV=$(git -C ${REPO_ROOT} rev-parse HEAD)

if git -C ${REPO_ROOT} diff --quiet HEAD; then
    DOCKER_LABEL=${GIT_REV}
else
    DOCKER_LABEL=${GIT_REV}-dirty
fi

DOCKER_IMAGE_TAG=${@:-mangatasolutions/rollup-node:local}
DOCKER_TAGS=""

echo "building docker image ${DOCKER_IMAGE_TAG} with tags:"
for tag in ${DOCKER_IMAGE_TAG}; do
    echo \t - ${tag}
    DOCKER_TAGS="${DOCKER_TAGS} -t ${tag}"
done

if [ ! -e ${BINARY_PATH} ]; then
    echo "env variable 'BUILD_DIR' : ${BINARY_PATH} not found" >&2
    exit 1
fi

docker build \
    --build-arg BINARY_PATH=${BINARY_PATH} \
    --label "git_rev=${DOCKER_LABEL}" \
    ${DOCKER_TAGS} \
    -f ${REPO_ROOT}/Dockerfile \
    .
