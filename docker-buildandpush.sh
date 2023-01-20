#!/bin/sh
set -e
USERNAME="virtualraven"
TAG=$(git describe --tags --match="v*.*" HEAD | sed -E 's/v([1-9][0-9]*\.[0-9]+)/\1/' --)
IMAGE_NAME="$USERNAME/webgallery-nginx:version-$TAG"
API_IMAGE_NAME="$USERNAME/webgallery-api:version-$TAG"
SOURCE_COMMIT="$(git rev-parse HEAD)"
    docker build --pull --build-arg SOURCE_COMMIT=$SOURCE_COMMIT -f Dockerfile -t $IMAGE_NAME .  \
&&  docker build --pull --build-arg SOURCE_COMMIT=$SOURCE_COMMIT -f Dockerfile --target website-backend -t $API_IMAGE_NAME . \
&&  docker push $IMAGE_NAME  \
&&  docker push $API_IMAGE_NAME 
