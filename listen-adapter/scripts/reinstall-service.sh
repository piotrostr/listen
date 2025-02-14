#!/bin/bash

git pull
cargo build --release
sudo systemctl stop listen-adapter.service
sudo ./scripts/install-service.sh
