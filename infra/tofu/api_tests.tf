locals {
  one_day = 86400
}

resource "cloudflare_r2_bucket" "api_tests" {
  account_id    = var.cloudflare_account_id
  name          = "fortropp-test"
  location      = "enam"
  storage_class = "Standard"
}

resource "cloudflare_r2_bucket_lifecycle" "api_tests" {
  account_id  = var.cloudflare_account_id
  bucket_name = cloudflare_r2_bucket.api_tests.name
  rules = [
    {
      id         = "expire-after-one-day"
      enabled    = true
      conditions = { prefix = "" }
      delete_objects_transition = {
        condition = { type = "Age", max_age = local.one_day }
      }
      abort_multipart_uploads_transition = {
        condition = { type = "Age", max_age = local.one_day }
      }
    }
  ]
}
