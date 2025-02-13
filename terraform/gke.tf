resource "google_container_cluster" "default" {
  name     = var.cluster_name
  location = var.region

  enable_autopilot         = true
  enable_l4_ilb_subsetting = true

  network    = google_compute_network.vpc.name
  subnetwork = google_compute_subnetwork.subnet.name

  ip_allocation_policy {
    stack_type                    = "IPV4_IPV6"
    services_secondary_range_name = "services"
    cluster_secondary_range_name  = "pods"
  }

  deletion_protection = false
}
