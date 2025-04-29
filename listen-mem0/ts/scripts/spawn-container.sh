#!/bin/bash

# Exit on any error
set -e

# Source the .env file
if [ -f .env ]; then
    source .env
    echo "Environment variables loaded from .env"
    echo "QDRANT_URL: ${QDRANT_URL}"
    echo "QDRANT_COLLECTION_NAME: ${QDRANT_COLLECTION_NAME}"
    echo "GEMINI_API_KEY: ${GEMINI_API_KEY:0:8}...${GEMINI_API_KEY: -4}"
else
    echo "No .env file found!"
    exit 1
fi

# Build the Docker image
echo "Building Docker image..."
docker build -t listen-mem0 .

# Stop and remove existing container if it exists
echo "Cleaning up any existing container..."
docker rm -f listen-mem0 2>/dev/null || true

# Run the container with explicit environment variables
echo "Starting container..."
docker run -d \
  --name listen-mem0 \
  --network host \
  -e QDRANT_URL="${QDRANT_URL}" \
  -e QDRANT_COLLECTION_NAME="${QDRANT_COLLECTION_NAME}" \
  -e GEMINI_API_KEY="${GEMINI_API_KEY}" \
  --restart unless-stopped \
  listen-mem0

# Show container status
echo "Container started. Checking status..."
docker ps | grep listen-mem0

# Verify environment variables in container
echo "Verifying environment variables in container:"
docker exec listen-mem0 env | grep -E 'QDRANT|GEMINI' | sed 's/\(GEMINI_API_KEY=\)[^[:space:]]*/\1'${GEMINI_API_KEY:0:8}'...'${GEMINI_API_KEY: -4}'/'