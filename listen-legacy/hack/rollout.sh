#!/bin/bash

docker stack rm "trader_$(git rev-parse --short HEAD~)"

docker compose pull

docker stack deploy "trader_$(git rev-parse --short HEAD)" \
  --compose-file docker-compose.yml