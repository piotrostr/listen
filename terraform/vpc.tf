resource "google_compute_network" "vpc" {
  name                     = "listen-network"
  auto_create_subnetworks  = false
  enable_ula_internal_ipv6 = true
  project                  = var.project_id
}

resource "google_compute_subnetwork" "subnet" {
  name          = "listen-subnet"
  ip_cidr_range = "10.0.0.0/20"
  region        = var.region
  network       = google_compute_network.vpc.id
  project       = var.project_id

  stack_type       = "IPV4_IPV6"
  ipv6_access_type = "EXTERNAL"


  secondary_ip_range {
    range_name    = "pods"
    ip_cidr_range = "10.1.0.0/16"
  }

  secondary_ip_range {
    range_name    = "services"
    ip_cidr_range = "10.2.0.0/20"
  }
}
