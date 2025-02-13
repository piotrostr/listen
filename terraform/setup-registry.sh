#!/bin/bash

# Get the current user's email
USER_EMAIL=$(gcloud config get-value account)
if [ -z "$USER_EMAIL" ]; then
    echo "Please run 'gcloud auth login' first"
    exit 1
fi

# Enable required APIs
gcloud services enable \
    containerregistry.googleapis.com \
    cloudbuild.googleapis.com

# Configure Docker to use gcloud as a credential helper
gcloud auth configure-docker

# Create a service account for Cloud Build if it doesn't exist
SA_EMAIL="cloud-build@listen-sol-prod.iam.gserviceaccount.com"
if ! gcloud iam service-accounts describe "$SA_EMAIL" >/dev/null 2>&1; then
    gcloud iam service-accounts create cloud-build \
        --display-name="Cloud Build Service Account"
fi

# Grant necessary permissions
gcloud projects add-iam-policy-binding listen-sol-prod \
    --member="serviceAccount:$SA_EMAIL" \
    --role="roles/storage.admin"

gcloud projects add-iam-policy-binding listen-sol-prod \
    --member="serviceAccount:$SA_EMAIL" \
    --role="roles/cloudbuild.builds.builder"

# Grant permissions to the user
gcloud projects add-iam-policy-binding listen-sol-prod \
    --member="user:$USER_EMAIL" \
    --role="roles/cloudbuild.builds.editor"

gcloud projects add-iam-policy-binding listen-sol-prod \
    --member="user:$USER_EMAIL" \
    --role="roles/storage.admin"

echo "Container Registry setup complete!" 