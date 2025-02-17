#!/bin/bash

# Exit on any error
set -e

# Create system user if it doesn't exist
if ! id -u listen-engine > /dev/null 2>&1; then
    sudo useradd -r -s /bin/false listen-engine
fi

# Create necessary directories
sudo mkdir -p /opt/listen-engine
sudo mkdir -p /etc/listen-engine

# Copy binary from target/release to /usr/local/bin
sudo cp target/release/engine /usr/local/bin/
sudo chmod +x /usr/local/bin/engine

# Create environment file from .env
sudo cp .env /etc/listen-engine/environment

# Set proper ownership and permissions
sudo chown root:listen-engine /etc/listen-engine/environment
sudo chmod 640 /etc/listen-engine/environment
sudo chown -R listen-engine:listen-engine /opt/listen-engine

# Create systemd service file
sudo cp listen-engine.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable listen-engine
sudo systemctl start listen-engine

# Show status
echo "Service installation complete. Checking status..."
sudo systemctl status listen-engine 
