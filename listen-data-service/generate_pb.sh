#!/bin/bash

OUTPUT_PATH=ao-solana-dex-trades-pubsub-v0.0.3.spkg

curl https://spkg.io/v1/files/ao-solana-dex-trades-pubsub-v0.0.3.spkg \
    --output $OUTPUT_PATH

buf generate --exclude-path="google" ./$OUTPUT_PATH#format=bin
