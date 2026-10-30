provider "cloudflare" {}

provider "cloudflare" {
  alias     = "email_routing"
  api_token = var.cloudflare_email_routing_token
}

provider "cloudflare" {
  alias     = "turnstile"
  api_token = var.cloudflare_turnstile_token
}

provider "digitalocean" {}
