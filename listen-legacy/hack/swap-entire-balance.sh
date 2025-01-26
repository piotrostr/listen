#!/bin/bash

# usage ./swap-entire-balance.sh <input-mint>
INPUT_MINT=$1

./target/release/listen --url $RPC_URL \
swap \
--input-mint $INPUT_MINT \
--output-mint sol