#!/bin/bash

# Exit on any error
set -e

echo "Stopping and disabling systemd service..."
sudo systemctl stop listen-0mem || true
sudo systemctl disable listen-0mem || true

echo "Removing service files..."
sudo rm -f /etc/systemd/system/listen-0mem.service
sudo systemctl daemon-reload

echo "Removing application files..."
sudo rm -rf /opt/listen-0mem
sudo rm -rf /etc/listen-0mem

echo "Removing system user..."
sudo userdel listen-0mem || true

echo "Service completely removed." 