#!/bin/bash

# Exit on any error
set -e

# Create system user if it doesn't exist
if ! id -u listen-0mem > /dev/null 2>&1; then
    sudo useradd -r -s /bin/false listen-0mem
fi

# Create necessary directories
sudo mkdir -p /opt/listen-0mem
sudo mkdir -p /etc/listen-0mem

# Copy application files
sudo cp -r ./* /opt/listen-0mem/

# Copy environment file
sudo cp .env /etc/listen-0mem/environment

# Set proper ownership and permissions
sudo chown root:listen-0mem /etc/listen-0mem/environment
sudo chmod 640 /etc/listen-0mem/environment
sudo chown -R listen-0mem:listen-0mem /opt/listen-0mem

# Copy systemd service file
sudo cp listen-0mem.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable listen-0mem
sudo systemctl start listen-0mem

# Show status
echo "Service installation complete. Checking status..."
sudo systemctl status listen-0mem 