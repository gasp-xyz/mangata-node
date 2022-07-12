#!/bin/bash -x
REPO_ROOT=$(readlink -f $(dirname $(readlink -f $0)))
OUTPUT_DIR=docker-cargo/
CARGO_HOME=${CARGO_HOME:-$HOME/.cargo}


DOCKER_BUILDER_IMAGE=${DOCKER_BUILDER_IMAGE:-mangatasolutions/node-builder:0.1}
DOCKER_USER="$(id -u):$(id -g)"
DOCKER_JOB_NAME=cargo-wrapper
if [ -n "${DISABLE_TTY}" ]; then
    ALLOCATE_TTY_OR_NOT="-i"
else
    ALLOCATE_TTY_OR_NOT="-it"
fi

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
	echo "docker image ${DOCKER_BUILDER_IMAGE} not found - pulling" >&2
	docker pull ${DOCKER_BUILDER_IMAGE}
fi

if [ -e ${CARGO_HOME} ] ; then
    CARGO_CACHE_GIT=${CARGO_HOME}/git
    CARGO_CACHE_REGISTRY=${CARGO_HOME}/registry

    if [ ! -e ${CARGO_CACHE_GIT} ]; then
        mkdir -p ${CARGO_CACHE_GIT}
    fi

    if [ ! -e ${CARGO_CACHE_REGISTRY} ]; then
        mkdir -p ${CARGO_CACHE_REGISTRY}
    fi
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

if [ -n "${DISABLE_CARGO_CACHE}" ]; then
    DOCKER_MOUNT_CACHE_VOLUMES=""
else
    DOCKER_MOUNT_CACHE_VOLUMES="-v ${CARGO_CACHE_GIT}:/opt/cargo/git -v ${CARGO_CACHE_REGISTRY}:/opt/cargo/registry"
fi

docker run apt install -y cmake

docker run \
	--rm \
	--name=${DOCKER_JOB_NAME} \
	--user $DOCKER_USER \
	-v ${REPO_ROOT}:/code \
        ${DOCKER_MOUNT_CACHE_VOLUMES} \
        ${DOCKER_RUN_EXTRA_ARGS} \
	-e CARGO_TARGET_DIR="/code/${OUTPUT_DIR}" \
	${ALLOCATE_TTY_OR_NOT} ${DOCKER_BUILDER_IMAGE} \
	cargo ${CARGO_COMMAND} --manifest-path=/code/Cargo.toml ${CARGO_ARGS}
