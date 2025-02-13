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
  default     = "listen-cluster"
}

variable "rpc_url" {
  description = "Solana RPC URL"
  type        = string
  sensitive   = true
}

// optional
variable "ws_url" {
  description = "Solana WebSocket URL"
  type        = string
  sensitive   = true
}

variable "geyser_url" {
  description = "The Geyser URL"
  type        = string
  sensitive   = true
}

variable "geyser_x_token" {
  description = "The Geyser X-Header Token"
  type        = string
  sensitive   = true
} 
