locals {
  thirty_days = 2592000
}

resource "cloudflare_r2_bucket" "ci_cache" {
  account_id    = var.cloudflare_account_id
  name          = "fortropp-ci-cache"
  location      = "enam"
  storage_class = "Standard"
}

resource "cloudflare_r2_bucket_lifecycle" "ci_cache" {
  account_id  = var.cloudflare_account_id
  bucket_name = cloudflare_r2_bucket.ci_cache.name
  rules = [
    {
      id         = "expire-after-thirty-days"
      enabled    = true
      conditions = { prefix = "" }
      delete_objects_transition = {
        condition = { type = "Age", max_age = local.thirty_days }
      }
      abort_multipart_uploads_transition = {
        condition = { type = "Age", max_age = local.one_day }
      }
    }
  ]
}
