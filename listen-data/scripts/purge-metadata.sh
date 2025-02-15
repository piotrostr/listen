#!/bin/bash

redis-cli SCAN 0 MATCH "solana:metadata:*" | while read -r line; do
    if [[ $line =~ ^[0-9]+$ ]]; then
        cursor=$line
    else
        redis-cli DEL "$line"
    fi
done