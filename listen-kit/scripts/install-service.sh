#!/bin/bash

# Exit on any error
set -e

# Create system user if it doesn't exist
if ! id -u listen-kit > /dev/null 2>&1; then
    sudo useradd -r -s /bin/false listen-kit
fi

# Create necessary directories
sudo mkdir -p /opt/listen-kit
sudo mkdir -p /etc/listen-kit

# Copy binary from target/release to /usr/local/bin
sudo cp target/release/kit /usr/local/bin/
sudo chmod +x /usr/local/bin/kit

# Create environment file from .env
sudo cp .env /etc/listen-kit/environment

# Set proper ownership and permissions
sudo chown root:listen-kit /etc/listen-kit/environment
sudo chmod 640 /etc/listen-kit/environment
sudo chown -R listen-kit:listen-kit /opt/listen-kit

# Create systemd service file
sudo cp listen-kit.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable listen-kit
sudo systemctl start listen-kit

# Show status
echo "Service installation complete. Checking status..."
sudo systemctl status listen-kit 
