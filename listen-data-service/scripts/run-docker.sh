#!/bin/bash

TAG=$(git rev-parse --short HEAD)
IMAGE=$(docker images -q listen-data-service:"$TAG")

docker build -t "$IMAGE" . \
    && docker run -it --rm -v "$(pwd)"/.env:/app/.env "$IMAGE"
