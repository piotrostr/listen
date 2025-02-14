#!/bin/bash

git pull
cargo build --release
sudo systemctl stop listen-indexer.service
sudo ./scripts/install-service.sh
