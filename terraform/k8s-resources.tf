resource "kubernetes_namespace" "listen_data" {
  metadata {
    name = "listen-data"
  }
}

resource "kubernetes_deployment" "indexer" {
  metadata {
    name      = "indexer"
    namespace = kubernetes_namespace.listen_data.metadata[0].name
  }

  spec {
    replicas = 1
    strategy {
      type = "Recreate"
    }

    selector {
      match_labels = {
        app = "indexer"
      }
    }

    template {
      metadata {
        labels = {
          app = "indexer"
        }
      }

      spec {
        container {
          name  = "indexer"
          image = "gcr.io/listen-sol-prod/listen-indexer:latest"

          resources {
            requests = {
              cpu    = "2"
              memory = "4Gi"
            }
            limits = {
              cpu    = "4"
              memory = "8Gi"
            }
          }

          env {
            name  = "REDIS_URL"
            value = "redis://redis-service:6379"
          }

          env {
            name  = "CLICKHOUSE_USERNAME"
            value = "default"
          }

          env {
            name  = "CLICKHOUSE_PASSWORD"
            value = "default"
          }

          env {
            name  = "CLICKHOUSE_DATABASE"
            value = "default"
          }

          env {
            name  = "CLICKHOUSE_URL"
            value = "http://clickhouse-service:8123"
          }

          env {
            name = "RPC_URL"
            value_from {
              secret_key_ref {
                name = kubernetes_secret.indexer_secrets.metadata[0].name
                key  = "RPC_URL"
              }
            }
          }

          env {
            name = "WS_URL"
            value_from {
              secret_key_ref {
                name = kubernetes_secret.indexer_secrets.metadata[0].name
                key  = "WS_URL"
              }
            }
          }

          env {
            name = "GEYSER_URL"
            value_from {
              secret_key_ref {
                name = kubernetes_secret.indexer_secrets.metadata[0].name
                key  = "GEYSER_URL"
              }
            }
          }

          env {
            name = "GEYSER_X_TOKEN"
            value_from {
              secret_key_ref {
                name = kubernetes_secret.indexer_secrets.metadata[0].name
                key  = "GEYSER_X_TOKEN"
              }
            }
          }
        }
      }
    }
  }
}

resource "kubernetes_secret" "indexer_secrets" {
  metadata {
    name      = "indexer-secrets"
    namespace = kubernetes_namespace.listen_data.metadata[0].name
  }

  data = {
    RPC_URL        = var.rpc_url
    WS_URL         = var.ws_url
    GEYSER_URL     = var.geyser_url
    GEYSER_X_TOKEN = var.geyser_x_token
  }
}

resource "kubernetes_deployment" "redis" {
  metadata {
    name      = "redis"
    namespace = kubernetes_namespace.listen_data.metadata[0].name
  }

  spec {
    replicas = 1
    strategy {
      type = "Recreate"
    }

    selector {
      match_labels = {
        app = "redis"
      }
    }

    template {
      metadata {
        labels = {
          app = "redis"
        }
      }

      spec {
        container {
          name  = "redis"
          image = "redis:latest"

          resources {
            requests = {
              cpu    = "500m"
              memory = "1Gi"
            }
            limits = {
              cpu    = "2"
              memory = "1.5Gi"
            }
          }

          volume_mount {
            name       = "redis-config"
            mount_path = "/usr/local/etc/redis/redis.conf"
            sub_path   = "redis.conf"
          }

          volume_mount {
            name       = "redis-data"
            mount_path = "/data"
          }
        }

        volume {
          name = "redis-config"
          config_map {
            name = kubernetes_config_map.redis_config.metadata[0].name
          }
        }

        volume {
          name = "redis-data"
          persistent_volume_claim {
            claim_name = kubernetes_persistent_volume_claim.redis_data.metadata[0].name
          }
        }
      }
    }
  }
}

resource "kubernetes_deployment" "clickhouse" {
  metadata {
    name      = "clickhouse"
    namespace = kubernetes_namespace.listen_data.metadata[0].name
  }

  spec {
    replicas = 1
    strategy {
      type = "Recreate"
    }

    selector {
      match_labels = {
        app = "clickhouse"
      }
    }

    template {
      metadata {
        labels = {
          app = "clickhouse"
        }
      }

      spec {
        container {
          name  = "clickhouse"
          image = "clickhouse/clickhouse-server:latest"

          resources {
            requests = {
              cpu    = "1"
              memory = "2Gi"
            }
            limits = {
              cpu    = "2"
              memory = "2Gi"
            }
          }

          env {
            name  = "CLICKHOUSE_PASSWORD"
            value = "default"
          }

          volume_mount {
            name       = "clickhouse-data"
            mount_path = "/var/lib/clickhouse"
          }

          volume_mount {
            name       = "clickhouse-logs"
            mount_path = "/var/log/clickhouse-server"
          }

          volume_mount {
            name       = "clickhouse-config"
            mount_path = "/etc/clickhouse-server/config.xml"
            sub_path   = "config.xml"
          }

          volume_mount {
            name       = "clickhouse-users"
            mount_path = "/etc/clickhouse-server/users.xml"
            sub_path   = "users.xml"
          }
        }

        volume {
          name = "clickhouse-data"
          persistent_volume_claim {
            claim_name = kubernetes_persistent_volume_claim.clickhouse_data.metadata[0].name
          }
        }

        volume {
          name = "clickhouse-logs"
          persistent_volume_claim {
            claim_name = kubernetes_persistent_volume_claim.clickhouse_logs.metadata[0].name
          }
        }

        volume {
          name = "clickhouse-config"
          config_map {
            name = kubernetes_config_map.clickhouse_config.metadata[0].name
          }
        }

        volume {
          name = "clickhouse-users"
          config_map {
            name = kubernetes_config_map.clickhouse_users.metadata[0].name
          }
        }
      }
    }
  }
}

resource "kubernetes_deployment" "adapter" {
  metadata {
    name      = "adapter"
    namespace = kubernetes_namespace.listen_data.metadata[0].name
  }

  spec {
    replicas = 1
    strategy {
      type = "Recreate"
    }

    selector {
      match_labels = {
        app = "adapter"
      }
    }

    template {
      metadata {
        labels = {
          app = "adapter"
        }
      }

      spec {
        container {
          name  = "adapter"
          image = "gcr.io/listen-sol-prod/listen-adapter:latest"

          resources {
            requests = {
              cpu    = "500m"
              memory = "1Gi"
            }
            limits = {
              cpu    = "1"
              memory = "2Gi"
            }
          }

          env {
            name  = "REDIS_URL"
            value = "redis://redis-service:6379"
          }
          env {
            name  = "HOST"
            value = "0.0.0.0"
          }
          env {
            name  = "PORT"
            value = "6968"
          }
        }
      }
    }
  }
}

# Services
resource "kubernetes_service" "redis" {
  metadata {
    name      = "redis-service"
    namespace = kubernetes_namespace.listen_data.metadata[0].name
  }

  spec {
    selector = {
      app = "redis"
    }

    port {
      port        = 6379
      target_port = 6379
    }

    type = "ClusterIP"
  }
}

resource "kubernetes_service" "clickhouse" {
  metadata {
    name      = "clickhouse-service"
    namespace = kubernetes_namespace.listen_data.metadata[0].name
  }

  spec {
    selector = {
      app = "clickhouse"
    }

    port {
      name        = "http"
      port        = 8123
      target_port = 8123
    }

    port {
      name        = "native"
      port        = 9000
      target_port = 9000
    }

    type = "ClusterIP"
  }
}

resource "kubernetes_service" "adapter" {
  metadata {
    name      = "adapter-service"
    namespace = kubernetes_namespace.listen_data.metadata[0].name
  }

  spec {
    selector = {
      app = "adapter"
    }

    port {
      port        = 6968
      target_port = 6968
    }

    type = "ClusterIP"
  }
}

# Ingress
resource "kubernetes_ingress_v1" "adapter_ingress" {
  metadata {
    name      = "adapter-ingress"
    namespace = kubernetes_namespace.listen_data.metadata[0].name
    annotations = {
      "kubernetes.io/ingress.class"                 = "gce"
      "kubernetes.io/ingress.global-static-ip-name" = google_compute_global_address.adapter_ip.name
    }
  }

  spec {
    rule {
      http {
        path {
          path = "/*"
          backend {
            service {
              name = kubernetes_service.adapter.metadata[0].name
              port {
                number = 6968
              }
            }
          }
        }
      }
    }
  }
}

resource "google_compute_global_address" "adapter_ip" {
  name = "adapter-ip"
}

# Add ConfigMaps for configurations
resource "kubernetes_config_map" "redis_config" {
  metadata {
    name      = "redis-config"
    namespace = kubernetes_namespace.listen_data.metadata[0].name
  }

  data = {
    "redis.conf" = file("${path.module}/../listen-data-service/redis/redis.conf")
  }
}

resource "kubernetes_config_map" "clickhouse_config" {
  metadata {
    name      = "clickhouse-config"
    namespace = kubernetes_namespace.listen_data.metadata[0].name
  }

  data = {
    "config.xml" = file("${path.module}/../listen-data-service/clickhouse/config.xml")
  }
}

resource "kubernetes_config_map" "clickhouse_users" {
  metadata {
    name      = "clickhouse-users"
    namespace = kubernetes_namespace.listen_data.metadata[0].name
  }

  data = {
    "users.xml" = file("${path.module}/../listen-data-service/clickhouse/users.xml")
  }
}

resource "kubernetes_persistent_volume_claim" "redis_data" {
  metadata {
    name      = "redis-data"
    namespace = kubernetes_namespace.listen_data.metadata[0].name
  }

  spec {
    access_modes = ["ReadWriteOnce"]
    resources {
      requests = {
        storage = "10Gi"
      }
    }
  }
}

resource "kubernetes_persistent_volume_claim" "clickhouse_data" {
  metadata {
    name      = "clickhouse-data"
    namespace = kubernetes_namespace.listen_data.metadata[0].name
  }

  spec {
    access_modes = ["ReadWriteOnce"]
    resources {
      requests = {
        storage = "50Gi"
      }
    }
  }
}

resource "kubernetes_persistent_volume_claim" "clickhouse_logs" {
  metadata {
    name      = "clickhouse-logs"
    namespace = kubernetes_namespace.listen_data.metadata[0].name
  }

  spec {
    access_modes = ["ReadWriteOnce"]
    resources {
      requests = {
        storage = "10Gi"
      }
    }
  }
} 