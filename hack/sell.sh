#!/bin/bash

curl -X POST localhost:8081/sell-simple \
  -H "Content-Type: application/json" \
  -H "Accept: application/json" \
  -d '{ 
  "amm_pool": "HjuhKVbvqaUTA9k3vEdyYJz4qEcTQh4Vg8s36eHBXR5Z",
  "lamports_spent": 10000000
}'
