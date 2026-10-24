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
