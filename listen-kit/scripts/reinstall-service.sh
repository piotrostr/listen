#!/bin/bash

git pull
cargo build --release
sudo systemctl stop listen-kit.service
sudo ./scripts/install-service.sh
