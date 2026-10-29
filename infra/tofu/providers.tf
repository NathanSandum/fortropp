provider "cloudflare" {}

provider "cloudflare" {
  alias     = "email_routing"
  api_token = var.cloudflare_email_routing_token
}

provider "digitalocean" {}
