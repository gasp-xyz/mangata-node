#!/bin/bash
REPO_ROOT=$(readlink -f $(dirname $(readlink -f $0)))
ARTIFACTS_DIR=$REPO_ROOT/docker-artifacts
BUILD_CACHE_DIR=docker-build/cache
BUILD_OUTPUT_DIR=docker-build/
CARGO_HOME=${CARGO_HOME:-$HOME/.cargo}


DOCKER_BUILDER_IMAGE=${DOCKER_BUILDER_IMAGE:-mangatasolutions/node-builder:0.1}
DOCKER_USER="$(id -u):$(id -g)"
DOCKER_JOB_NAME=cargo-wrapper
CARGO_COMMAND=$1
CARGO_ARGS=${@:2}

if [ "$CARGO_COMMAND" == "kill" ]; then
    docker kill ${DOCKER_JOB_NAME}
    exit 0
fi

if [ "$CARGO_COMMAND" == "clean" ]; then
	rm -rf  ${REPO_ROOT}/${BUILD_CACHE_DIR}
	rm -rf  ${REPO_ROOT}/${BUILD_OUTPUT_DIR}
	mkdir -p ${REPO_ROOT}/${BUILD_OUTPUT_DIR}
	mkdir -p ${REPO_ROOT}/${BUILD_CACHE_DIR}
fi

if ! which docker > /dev/null; then
	echo "docker not installed" >&2
	exit -1
fi

if docker inspect ${DOCKER_BUILDER_IMAGE} > /dev/null; then
	echo "building using docker image ${DOCKER_BUILDER_IMAGE}"
else
	echo "docker image ${DOCKER_BUILDER_IMAGE} not found" >&2
	exit -1
fi

# signal_handler () 
# {
#   docker kill ${DOCKER_JOB_NAME}
#   exit 0
# }
#
# trap signal_handler SIGKILL SIGINT SIGTERM

	# -e CARGO_HOME="/code/${BUILD_CACHE_DIR}" \

if [ -e ${CARGO_HOME} ]; then
    echo "using local cargo caches from ${CARGO_HOME}"
    MOUNT_LOCAL_CACHE="-v ${CARGO_HOME}/git:/opt/cargo/git -v ${CARGO_HOME}/registry:/opt/cargo/registry"
else
    echo "cache not available"
fi

docker run \
	--rm \
	--name=${DOCKER_JOB_NAME} \
	--user $DOCKER_USER \
	-v ${REPO_ROOT}:/code \
        ${MOUNT_LOCAL_CACHE} \
	-e CARGO_TARGET_DIR="/code/${BUILD_OUTPUT_DIR}" \
	-it ${DOCKER_BUILDER_IMAGE} \
	cargo ${CARGO_COMMAND} --manifest-path=/code/Cargo.toml ${CARGO_ARGS}
