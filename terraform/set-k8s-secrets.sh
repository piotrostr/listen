#!/bin/bash

kubectl create secret generic indexer-secrets \
    --namespace=listen-data-service \
    --from-env-file=../listen-data-service/.env
