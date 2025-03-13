#!/usr/bin/env bash

trap "echo; exit" INT
trap "echo; exit" HUP

DOCKER_DEFAULT_PLATFORM=$DOCKER_DEFAULT_PLATFORM
# fallback to this default platform only if no platform argument specified
DOCKER_FALLBACK_PLATFORM=linux/amd64
echo ${DOCKER_DEFAULT_PLATFORM:=$DOCKER_FALLBACK_PLATFORM}

# if they call this script from project root or from within docker/ folder then
# in both cases the PARENT_DIR will refer to the project root. it supports calls from symlinks
PROJECT_ROOT=$(dirname "$(dirname "$(realpath "${BASH_SOURCE[0]}")")")

# verify that we did infact successfully changed to the project root directory
if [ -e ./docker/Dockerfile ]
then
  echo "already in project root"
else
	echo "switching to project root"
  cd $PROJECT_ROOT
fi

# map the docker socket between containers to prevent error
# `Error: Cannot connect to the Docker daemon`
# since otherwise without mapping it will use
# `unix:///home/rouanubuntu/.docker/desktop/docker.sock` only in
# the Docker container
try_run() {
  PLATFORM=$1
  printf "Running Docker container for $PLATFORM architecture.\n\n"

  docker run \
    --privileged \
    --platform $PLATFORM \
    --hostname maciejhirsz-kobold \
    --name maciejhirsz-kobold \
    --memory 750M \
    --memory-reservation 125M \
    --memory-swap 15G \
    --cpus 1 \
    --publish 0.0.0.0:5000:5000 \
    --volume ${PROJECT_ROOT}:/kobold:rw \
    --volume /var/run/docker.sock:/var/run/docker.sock \
    --rm -it -d maciejhirsz/kobold
}

if [[ "$OSTYPE" == "darwin"* ]]; then
  set -e

  echo "Mac OS (Darwin) detected."
  if [[ $(uname -m) == 'arm64' ]]; then
    echo "Detected Apple Silicon"
    # emulate using `linux/x86_64` for Apple Silicon to avoid error with `/lib64/ld-linux-x86-64.so.2`
    DOCKER_DEFAULT_PLATFORM=linux/x86_64
    try_run $DOCKER_DEFAULT_PLATFORM
  else
    try_run $DOCKER_DEFAULT_PLATFORM
  fi
else
  try_run $DOCKER_DEFAULT_PLATFORM
fi
