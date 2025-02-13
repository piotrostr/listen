#!/bin/bash

# Exit on any error
set -e

# Get the current git commit hash
COMMIT_SHA=$(git rev-parse --short HEAD)

# Project ID
PROJECT_ID="listen-sol-prod"

# Image name and tag
IMAGE_NAME="listen-data-service"
IMAGE_PATH="gcr.io/${PROJECT_ID}/${IMAGE_NAME}"

echo "Building and pushing ${IMAGE_PATH}:${COMMIT_SHA}"

# Build using Cloud Build
gcloud builds submit \
    --project="${PROJECT_ID}" \
    --config=cloudbuild.yaml \
    --substitutions=COMMIT_SHA="${COMMIT_SHA}" \
    .

echo "Build and push complete!"
echo "Image: ${IMAGE_PATH}:${COMMIT_SHA}"
echo "Latest tag: ${IMAGE_PATH}:latest" 