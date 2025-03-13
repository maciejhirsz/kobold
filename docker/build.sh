#!/usr/bin/env bash

# reference: https://github.com/paritytech/polkadot-sdk/pull/4450/files#top
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

GITUSER=maciejhirsz
GITREPO=kobold

# Build the image
echo "Building ${GITUSER}/${GITREPO}:latest docker image"
# improve the docker logs to actually allow debugging with BuildKit enabled since build time may be long
export BUILDKIT_PROGRESS=plain
export DOCKER_BUILDKIT=1

# note: optionally use `docker build --no-cache ...`
try_build() {
  echo "Building for $1 architecture"
  PLATFORM=$1
# heredoc for try/catch syntax cannot be indented. https://stackoverflow.com/a/25554904/3208553
bash -e << TRY
  time docker build \
    --platform $PLATFORM \
    -f $PROJECT_ROOT/docker/Dockerfile \
    -t ${GITUSER}/${GITREPO}:latest ./
TRY
  if [ $? -ne 0 ]; then
    printf "\n*** Detected error running 'docker build'. Trying 'docker buildx' instead...\n"
# heredoc cannot be indented
bash -e << TRY
  time docker buildx build \
    --platform $PLATFORM \
    -f $PROJECT_ROOT/docker/Dockerfile \
    -t ${GITUSER}/${GITREPO}:latest ./
TRY
    if [ $? -ne 0 ]; then
      printf "\n*** Detected unknown error running 'docker buildx'.\n"
      exit 1
    fi
  fi
}

# reference: https://github.com/paritytech/scripts/blob/master/get-substrate.sh
if [[ "$OSTYPE" == "darwin"* ]]; then
  set -e

  echo "Mac OS (Darwin) detected."
  if [[ $(uname -m) == 'arm64' ]]; then
    echo "Detected Apple Silicon"
    # emulate using `linux/x86_64` for Apple Silicon to avoid error with `/lib64/ld-linux-x86-64.so.2`
    DOCKER_DEFAULT_PLATFORM=linux/x86_64
    try_build $DOCKER_DEFAULT_PLATFORM
  else
    try_build $DOCKER_DEFAULT_PLATFORM
  fi
else
  try_build $DOCKER_DEFAULT_PLATFORM
fi

VERSION=0.1
docker tag ${GITUSER}/${GITREPO}:latest ${GITUSER}/${GITREPO}:v${VERSION}

# Show the list of available images for this repo
echo "Image is ready"
docker images | grep ${GITREPO}

if [ $? -ne 0 ]; then
  kill "$PPID"; exit 1;
fi
CONTAINER_ID=$(docker ps -n=1 -q)
printf "\nFinished building Docker container ${CONTAINER_ID}.\n\n"
