#!/bin/bash

# Exit on any error
set -e

# Check if nginx is installed
if ! command -v nginx >/dev/null 2>&1; then
    echo "Nginx is not installed. Please install nginx first."
    exit 1
fi

# Add nginx user if it doesn't exist
if ! id "nginx" &>/dev/null; then
    echo "Creating nginx user..."
    sudo useradd -r -s /sbin/nologin nginx
else
    echo "Nginx user already exists"
fi

# Check if required files exist in current directory
if [ ! -f "./nginx.conf" ]; then
    echo "Error: nginx.conf not found in current directory"
    exit 1
fi

# Stop nginx if it's running
if sudo systemctl is-active --quiet nginx; then
    echo "Stopping nginx service..."
    sudo systemctl stop nginx
fi

echo "Copying configuration files..."
sudo cp ./nginx.conf /etc/nginx/nginx.conf

echo "Validating nginx configuration..."
if ! sudo nginx -t; then
    echo "Error: Invalid nginx configuration"
    exit 1
fi

echo "Enabling and starting nginx service..."
sudo systemctl enable nginx
sudo systemctl restart nginx

echo "Setup completed successfully!"
