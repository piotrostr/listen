terraform {
  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "6.14.0"
    }
    google-beta = {
      source  = "hashicorp/google-beta"
      version = "6.14.0"
    }
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "2.23.0"
    }
  }
}

provider "google" {
  project = var.project_id
  region  = var.region
}

data "google_client_config" "default" {}

provider "kubernetes" {
  host                   = "https://${resource.google_container_cluster.default.endpoint}"
  token                  = data.google_client_config.default.access_token
  cluster_ca_certificate = base64decode(resource.google_container_cluster.default.master_auth.0.cluster_ca_certificate)
}
