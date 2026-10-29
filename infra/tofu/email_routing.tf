resource "cloudflare_email_routing_dns" "app" {
  provider = cloudflare.email_routing
  zone_id  = data.cloudflare_zone.app.zone_id
}

resource "cloudflare_email_routing_address" "operator" {
  provider   = cloudflare.email_routing
  account_id = var.cloudflare_account_id
  email      = var.operator_email
}

resource "cloudflare_email_routing_rule" "operator" {
  for_each = toset(["nathan", "support", "dmarc"])

  provider = cloudflare.email_routing
  zone_id  = data.cloudflare_zone.app.zone_id
  name     = "${each.key}@fortropp.dev"
  enabled  = true
  matchers = [
    {
      type  = "literal"
      field = "to"
      value = "${each.key}@fortropp.dev"
    }
  ]
  actions = [
    {
      type  = "forward"
      value = [cloudflare_email_routing_address.operator.email]
    }
  ]

  depends_on = [cloudflare_email_routing_dns.app]
}
