module "vpc" {
  source  = "terraform-google-modules/network/google"

  project_id   = var.project_id
  network_name = "listen-data-network"
  routing_mode = "REGIONAL"

  subnets = [
    {
      subnet_name   = "listen-data-subnet"
      subnet_ip     = "10.0.0.0/20"
      subnet_region = var.region
    }
  ]

  secondary_ranges = {
    listen-data-subnet = [
      {
        range_name    = "pods"
        ip_cidr_range = "10.1.0.0/16"
      },
      {
        range_name    = "services"
        ip_cidr_range = "10.2.0.0/20"
      }
    ]
  }
} 