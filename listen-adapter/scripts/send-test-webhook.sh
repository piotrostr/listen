#!/bin/bash

curl -X POST http://localhost:6968/webhook \
  -H "Content-Type: application/json" \
  -d '{
    "event": "transaction.confirmed",
    "transaction_id": "fmfdj6yqly31huorjqzq38zc",
    "wallet_id": "cm4db8x9t000ccn87pctvcg9j",
    "transaction_hash": "0x2446f1fd773fbb9f080e674b60c6a033c7ed7427b8b9413cf28a2a4a6da9b56c",
    "chain_id": "eip155:8453"
  }'
