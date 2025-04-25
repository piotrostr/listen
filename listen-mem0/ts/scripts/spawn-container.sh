#!/bin/bash

# Exit on any error
set -e

# Build the Docker image
echo "Building Docker image..."
docker build -t listen-mem0 .

# Stop and remove existing container if it exists
echo "Cleaning up any existing container..."
docker rm -f listen-mem0 2>/dev/null || true

# Run the container
echo "Starting container..."
docker run -d \
  --name listen-mem0 \
  --network host \
  --env-file .env \
  --restart unless-stopped \
  listen-mem0

# Show container status
echo "Container started. Checking status..."
docker ps | grep listen-mem0 