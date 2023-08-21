#!/bin/bash -x
REPO_ROOT=$(readlink -f $(dirname $(readlink -f $0)))
CODE_ROOT=${CODE_ROOT:-${REPO_ROOT}}
OUTPUT_DIR=docker-cargo/
CARGO_HOME=${CARGO_HOME:-$HOME/.cargo}

DOCKER_BUILDER_IMAGE=${DOCKER_BUILDER_IMAGE:-mangatasolutions/node-builder:multi}
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

if ! which docker >/dev/null; then
  echo "docker not installed" >&2
  exit 1
fi

if docker inspect ${DOCKER_BUILDER_IMAGE} >/dev/null; then
  echo "building using docker image ${DOCKER_BUILDER_IMAGE}"
else
  echo "docker image ${DOCKER_BUILDER_IMAGE} not found - pulling" >&2
  docker pull ${DOCKER_BUILDER_IMAGE}
fi

if [ -n "${REUSE_LOCAL_CACHE}" ]; then
  if [ -e ${CARGO_HOME} ]; then
    CARGO_CACHE_GIT=${CARGO_HOME}/git
    CARGO_CACHE_REGISTRY=${CARGO_HOME}/registry

    if [ ! -e ${CARGO_CACHE_GIT} ]; then
      mkdir -p ${CARGO_CACHE_GIT}
    fi

    if [ ! -e ${CARGO_CACHE_REGISTRY} ]; then
      mkdir -p ${CARGO_CACHE_REGISTRY}
    fi
  else
    echo "CARGO_HOME not set" >&2
    exit 1
  fi
  if ! [ -e ${SCCACHE_DIR} ]; then
    echo "SCCACHE_DIR not set" >&2
    exit 1
  fi
else
  CARGO_CACHE_GIT=${REPO_ROOT}/${OUTPUT_DIR}/cache/cargo/git
  CARGO_CACHE_REGISTRY=${REPO_ROOT}/${OUTPUT_DIR}/cache/cargo/registry
  SCCACHE_DIR=${REPO_ROOT}/${OUTPUT_DIR}/cache/sccache
  if [ ! -e ${CARGO_CACHE_REGISTRY} ]; then
    mkdir -p ${CARGO_CACHE_REGISTRY}
  fi
  if [ ! -e ${CARGO_CACHE_GIT} ]; then
    mkdir -p ${CARGO_CACHE_GIT}
  fi
  if [ ! -e ${SCCACHE_DIR} ]; then
    mkdir -p ${SCCACHE_DIR}
  fi
fi

if [ -n "${DISABLE_CARGO_CACHE}" ]; then
  DOCKER_MOUNT_CACHE_VOLUMES=""
else
  DOCKER_MOUNT_CACHE_VOLUMES="-v ${CARGO_CACHE_GIT}:/usr/local/cargo/git -v ${CARGO_CACHE_REGISTRY}:/usr/local/cargo/registry -v ${SCCACHE_DIR}:/.cache/sccache"
fi

docker run \
  --rm \
  --name=${DOCKER_JOB_NAME} \
  --user $DOCKER_USER \
  -v ${CODE_ROOT}:/code \
  ${DOCKER_MOUNT_CACHE_VOLUMES} \
  ${DOCKER_RUN_EXTRA_ARGS} \
  -e CARGO_TARGET_DIR="/code/${OUTPUT_DIR}" \
  ${ALLOCATE_TTY_OR_NOT} ${DOCKER_BUILDER_IMAGE} \
  /bin/bash -c "cargo ${CARGO_COMMAND} --manifest-path=/code/Cargo.toml ${CARGO_ARGS} && sccache --show-stats"
