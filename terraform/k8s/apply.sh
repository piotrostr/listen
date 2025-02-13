#!/bin/bash

# Apply all configurations in order
kubectl apply -f 00-namespace.yaml
kubectl apply -f 02-redis.yaml
kubectl apply -f 03-clickhouse.yaml
kubectl apply -f 04-indexer.yaml
# kubectl apply -f 05-adapter.yaml
