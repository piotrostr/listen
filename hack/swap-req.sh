#!/bin/bash

curl localhost:8080/buy \
  -H "Content-Type: application/json" \
  -H "Accept: application/json" \
  -d '{ 
  "amm_pool": "EztYYEeSHbWJFhp4yDX8GBR8V8ozUxtC9q7TTJDfFWrw",
  "input_mint": "So11111111111111111111111111111111111111112",
  "output_mint": "2RKHP8AuBEFjMf76EeEtJSF2kNh6mFmKt48THiqCdG8V",
  "amount": 10000000
}'
