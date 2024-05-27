#!/bin/bash

TAG=git rev-parse --short HEAD

docker compose pull

docker service deploy "trader_$TAG" \
	--compose-file docker.compose.yml