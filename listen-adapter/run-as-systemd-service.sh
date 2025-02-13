#!/bin/bash

# Exit on any error
set -e

# Create system user if it doesn't exist
if ! id -u listen-adapter > /dev/null 2>&1; then
    sudo useradd -r -s /bin/false listen-adapter
fi

# Create necessary directories
sudo mkdir -p /opt/listen-adapter
sudo mkdir -p /etc/listen-adapter

# Copy binary from target/release to /usr/local/bin
sudo cp target/release/adapter /usr/local/bin/
sudo chmod +x /usr/local/bin/adapter

# Add capability to bind to privileged ports
sudo setcap 'cap_net_bind_service=+ep' /usr/local/bin/adapter

# Copy environment file if it doesn't exist
if [ ! -f /etc/listen-adapter/environment ]; then
    sudo cp environment /etc/listen-adapter/
fi

# Set proper ownership and permissions
sudo chown root:listen-adapter /etc/listen-adapter/environment
sudo chmod 640 /etc/listen-adapter/environment
sudo chown -R listen-adapter:listen-adapter /opt/listen-adapter

sudo cp listen-adapter.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable listen-adapter
sudo systemctl start listen-adapter

# Show status
echo "Service installation complete. Checking status..."
sudo systemctl status listen-adapter 