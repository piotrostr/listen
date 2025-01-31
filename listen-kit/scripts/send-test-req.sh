#!/bin/bash

curl -X POST http://localhost:6969/v1/stream \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "What is the wallet address of your yours?",
    "chat_history": [],
    "chain": "evm",
  }'

