module "gke" {
  source = "terraform-google-modules/kubernetes-engine/google//modules/beta-autopilot-private-cluster"

  project_id        = var.project_id
  name             = var.cluster_name
  region           = var.region
  network          = module.vpc.network_name
  subnetwork       = module.vpc.subnets_names[0]
  ip_range_pods    = "pods"
  ip_range_services = "services"

  enable_vertical_pod_autoscaling = true
} 