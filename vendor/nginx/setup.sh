#!/bin/bash

# Add nginx user if it doesn't exist
if ! id "nginx" &>/dev/null; then
    sudo useradd -r -s /sbin/nologin nginx
fi

# Copy nginx configuration to /etc
sudo cp ./nginx.conf /etc/nginx.conf

sudo systemctl enable nginx
sudo systemctl start nginx
