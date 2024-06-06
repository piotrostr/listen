#!/bin/bash

curl localhost:8081/sell-simple \
  -H "Content-Type: application/json" \
  -H "Accept: application/json" \
  -d "{\"amm_pool\":\"$1\"}"
