output "kubernetes_cluster_name" {
  value       = resource.google_container_cluster.default.name
  description = "GKE Cluster Name"
}

output "kubernetes_cluster_host" {
  value       = resource.google_container_cluster.default.endpoint
  description = "GKE Cluster Host"
}

output "adapter_ip_address" {
  value       = google_compute_global_address.adapter_ip.address
  description = "Global IP address for the adapter service"
} 
