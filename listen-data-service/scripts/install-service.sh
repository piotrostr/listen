#!/bin/bash

# Exit on any error
set -e

# Create system user if it doesn't exist
if ! id -u listen-indexer > /dev/null 2>&1; then
    sudo useradd -r -s /bin/false listen-indexer
fi

# Create necessary directories
sudo mkdir -p /opt/listen-indexer
sudo mkdir -p /etc/listen-indexer

# Copy binary from target/release to /usr/local/bin
sudo cp target/release/indexer /usr/local/bin/
sudo chmod +x /usr/local/bin/indexer

# Create environment file from .env
sudo cp .env /etc/listen-indexer/environment

# Set proper ownership and permissions
sudo chown root:listen-indexer /etc/listen-indexer/environment
sudo chmod 640 /etc/listen-indexer/environment
sudo chown -R listen-indexer:listen-indexer /opt/listen-indexer

# Create systemd service file
sudo cp listen-indexer.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable listen-indexer
sudo systemctl start listen-indexer

# Show status
echo "Service installation complete. Checking status..."
sudo systemctl status listen-indexer 