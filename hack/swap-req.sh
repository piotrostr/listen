#!/bin/bash

curl localhost:8080/buy \
  -H "Content-Type: application/json" \
  -H "Accept: application/json" \
  -d '{ 
  "amm_pool": "HjuhKVbvqaUTA9k3vEdyYJz4qEcTQh4Vg8s36eHBXR5Z",
  "input_mint": "So11111111111111111111111111111111111111112",
  "output_mint": "Fb5JePphHK1SXrtmeQ58aSHBbqgfosLrmbHknsA2BSfU",
  "amount": 1000000
}'
