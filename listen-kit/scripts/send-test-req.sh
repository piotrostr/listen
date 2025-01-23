#!/bin/bash

curl -X POST http://localhost:8080/stream \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "What is the public key of your wallet?",
    "chat_history": []
  }'

