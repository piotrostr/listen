variable "project_id" {
  description = "The project ID to host the cluster in"
  default     = "listen-sol-prod"
}

variable "region" {
  description = "The region to host the cluster in"
  default     = "europe-west3" # Frankfurt
}

variable "cluster_name" {
  description = "The name for the GKE cluster"
  default     = "listen-data-cluster"
} 