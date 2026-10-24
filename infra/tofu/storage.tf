locals {
  app_origin = "https://fortropp.dev"
}

resource "cloudflare_r2_bucket" "site_files" {
  account_id    = var.cloudflare_account_id
  name          = "fortropp-prod-enam"
  location      = "enam"
  storage_class = "Standard"
}

resource "cloudflare_r2_bucket_cors" "site_files" {
  account_id  = var.cloudflare_account_id
  bucket_name = cloudflare_r2_bucket.site_files.name
  rules = [
    {
      allowed = {
        methods = ["GET", "PUT", "HEAD"]
        origins = [local.app_origin]
        headers = ["*"]
      }
    }
  ]
}

resource "cloudflare_workers_kv_namespace" "site_routes" {
  account_id = var.cloudflare_account_id
  title      = "fortropp-prod"
}
