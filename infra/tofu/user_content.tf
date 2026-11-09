data "cloudflare_zone" "user_content" {
  filter = {
    name = "fortropp.app"
  }
}

locals {
  edge_hostname = "edge.fortropp.app"
  worker_only   = "100::"
}

resource "cloudflare_dns_record" "user_content" {
  for_each = toset(["fortropp.app", "*.fortropp.app", local.edge_hostname])

  zone_id = data.cloudflare_zone.user_content.zone_id
  name    = each.key
  type    = "AAAA"
  content = local.worker_only
  proxied = true
  ttl     = 1
}

resource "cloudflare_dns_record" "domain_targets" {
  zone_id = data.cloudflare_zone.user_content.zone_id
  name    = "*.${local.edge_hostname}"
  type    = "CNAME"
  content = local.edge_hostname
  proxied = true
  ttl     = 1
}

resource "cloudflare_dns_record" "apex_test" {
  zone_id = data.cloudflare_zone.user_content.zone_id
  name    = "qt.apex.fortropp.app"
  type    = "CNAME"
  content = local.edge_hostname
  proxied = true
  ttl     = 1
}

resource "cloudflare_workers_route" "user_content" {
  zone_id = data.cloudflare_zone.user_content.zone_id
  pattern = "*/*"
  script  = "fortropp-r2-gateway"
}

resource "cloudflare_custom_hostname_fallback_origin" "edge" {
  zone_id = data.cloudflare_zone.user_content.zone_id
  origin  = local.edge_hostname

  depends_on = [cloudflare_dns_record.user_content]
}
