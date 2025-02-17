#!/bin/bash

git pull
cargo build --release
sudo systemctl stop listen-engine.service
sudo ./scripts/install-service.sh
