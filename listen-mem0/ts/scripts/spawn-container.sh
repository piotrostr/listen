#!/bin/bash

# Exit on any error
set -e

# Build the Docker image
echo "Building Docker image..."
docker build -t listen-0mem .

# Stop and remove existing container if it exists
echo "Cleaning up any existing container..."
docker rm -f listen-0mem 2>/dev/null || true

# Run the container
echo "Starting container..."
docker run -d \
  --name listen-0mem \
  --network host \
  --env-file .env \
  --restart unless-stopped \
  listen-0mem

# Show container status
echo "Container started. Checking status..."
docker ps | grep listen-0mem 