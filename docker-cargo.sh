#!/bin/bash
REPO_ROOT=$(readlink -f $(dirname $(readlink -f $0)))
ARTIFACTS_DIR=$REPO_ROOT/docker-artifacts
OUTPUT_DIR=docker-build/
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

if [ -e ${CARGO_HOME} ]; then
    CARGO_CACHE_GIT=${CARGO_HOME}/git
    CARGO_CACHE_REGISTRY=${CARGO_HOME}/registry
else
    CARGO_CACHE_GIT=${REPO_ROOT}/${OUTPUT_DIR}/cargo_cache_git
    CARGO_CACHE_REGISTRY=${REPO_ROOT}/${OUTPUT_DIR}/cargo_cache_registry
    if [ ! -e ${CARGO_CACHE_REGISTRY} ]; then
        mkdir -p ${CARGO_CACHE_REGISTRY}
    fi
    if [ ! -e ${CARGO_CACHE_GIT} ]; then
        mkdir -p ${CARGO_CACHE_GIT}
    fi
fi

docker run \
	--rm \
	--name=${DOCKER_JOB_NAME} \
	--user $DOCKER_USER \
	-v ${REPO_ROOT}:/code \
        -v ${CARGO_CACHE_GIT}:/opt/cargo/git \
        -v ${CARGO_CACHE_REGISTRY}:/opt/cargo/registry \
	-e CARGO_TARGET_DIR="/code/${BUILD_OUTPUT_DIR}" \
	-i ${DOCKER_BUILDER_IMAGE} \
	cargo ${CARGO_COMMAND} --manifest-path=/code/Cargo.toml ${CARGO_ARGS}
