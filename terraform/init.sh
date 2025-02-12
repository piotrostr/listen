#!/bin/bash

PROJECT_ID="listen-sol-prod"

gcloud iam service-accounts create terraform-sa --display-name="Terraform Service Account"

gcloud projects add-iam-policy-binding $PROJECT_ID \
    --member="serviceAccount:terraform-sa@$PROJECT_ID.iam.gserviceaccount.com" \
    --role="roles/editor"

gcloud iam service-accounts keys create terraform-key.json \
    --iam-account=terraform-sa@$PROJECT_ID.iam.gserviceaccount.com
