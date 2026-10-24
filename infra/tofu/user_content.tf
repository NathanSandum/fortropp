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

resource "cloudflare_custom_hostname_fallback_origin" "edge" {
  zone_id = data.cloudflare_zone.user_content.zone_id
  origin  = local.edge_hostname

  depends_on = [cloudflare_dns_record.user_content]
}
