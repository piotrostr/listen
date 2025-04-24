#!/bin/bash

git pull
cd /opt/listen-0mem && sudo bun install
sudo systemctl stop listen-0mem.service
sudo ./scripts/install-service.sh 