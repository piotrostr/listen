#!/bin/bash

SPKG_PATH="tl-solana-dex-trades-1-0-22-v1.0.22.spkg"

buf generate --exclude-path="google" ./$SPKG_PATH#format=bin
