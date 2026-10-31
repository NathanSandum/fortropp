provider "cloudflare" {}

provider "cloudflare" {
  alias     = "email_routing"
  api_token = var.cloudflare_email_routing_token
}

provider "cloudflare" {
  alias     = "turnstile"
  api_token = var.cloudflare_turnstile_token
}

provider "cloudflare" {
  alias     = "waf"
  api_token = var.cloudflare_waf_token
}

provider "digitalocean" {}
