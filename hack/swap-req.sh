#!/bin/bash

# note this uses entire balance for swap
curl localhost:8081/sell \
  -H "Content-Type: application/json" \
  -H "Accept: application/json" \
  -d '{ 
  "amm_pool": "Da34D1CC7kkjXRvUM3EQ8DjjCcddW2igiVecEtsLwYSB",
  "input_mint": "6a2noYbcJE53tfpB4FugwUUwh9ME7JPXQuQ6zxr1KKNi",
  "output_mint": "So11111111111111111111111111111111111111112",
  "sol_vault": "3yugDWhJDbGStGU82vqCkn3QgkhUKMHYbKZjWS5kxPuj",
  "sol_pooled_when_bought": 15.0
}'
