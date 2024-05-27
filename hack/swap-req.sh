#!/bin/bash

# note this uses entire balance for swap
curl localhost:8081/sell \
  -H "Content-Type: application/json" \
  -H "Accept: application/json" \
  -d '{ 
  "amm_pool": "CEKbiXWBiPNX3wfYewKVq84f5nvV4dWDnAAm99icjvQB",
  "input_mint": "EPtjXZNFUmM6WgikVsQaXUoFuo2XdxwAJXcHktuBrNY5",
  "output_mint": "So11111111111111111111111111111111111111112",
  "sol_vault": "AkQoT45EEQeyjtdpVgcaZWgkd91tZ5ZPcYnJEuQK63bw",
  "sol_pooled_when_bought": 145
}'
