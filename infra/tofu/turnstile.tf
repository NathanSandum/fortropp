resource "cloudflare_turnstile_widget" "app" {
  account_id = var.cloudflare_account_id
  name       = "fortropp"
  domains    = ["fortropp.dev"]
  mode       = "managed"
}

output "turnstile_sitekey" {
  value = cloudflare_turnstile_widget.app.sitekey
}

output "turnstile_secret" {
  value     = cloudflare_turnstile_widget.app.secret
  sensitive = true
}
