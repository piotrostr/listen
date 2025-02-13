#!/bin/bash

# Delete all resources in reverse order
kubectl delete -f 04-adapter.yaml
kubectl delete -f 03-indexer.yaml
kubectl delete -f 02-clickhouse.yaml
kubectl delete -f 01-redis.yaml
kubectl delete -f 00-namespace.yaml

echo "Cleanup complete!" 