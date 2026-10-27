data "cloudflare_zone" "app" {
  filter = {
    name = "fortropp.dev"
  }
}

resource "cloudflare_dns_record" "app" {
  for_each = {
    A    = digitalocean_droplet.backend.ipv4_address
    AAAA = digitalocean_droplet.backend.ipv6_address
  }

  zone_id = data.cloudflare_zone.app.zone_id
  name    = "fortropp.dev"
  type    = each.key
  content = each.value
  proxied = true
  ttl     = 1
}

resource "cloudflare_zone_setting" "app_ssl" {
  zone_id    = data.cloudflare_zone.app.zone_id
  setting_id = "ssl"
  value      = "strict"
}

resource "tls_private_key" "app_origin" {
  algorithm   = "ECDSA"
  ecdsa_curve = "P256"
}

resource "tls_cert_request" "app_origin" {
  private_key_pem = tls_private_key.app_origin.private_key_pem
  dns_names       = ["fortropp.dev"]
}

resource "cloudflare_origin_ca_certificate" "app" {
  csr                = tls_cert_request.app_origin.cert_request_pem
  hostnames          = ["fortropp.dev"]
  request_type       = "origin-ecc"
  requested_validity = 5475
}
