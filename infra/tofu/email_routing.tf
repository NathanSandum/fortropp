resource "cloudflare_email_routing_dns" "app" {
  provider = cloudflare.email_routing
  zone_id  = data.cloudflare_zone.app.zone_id
}

resource "cloudflare_email_routing_address" "operator" {
  provider   = cloudflare.email_routing
  account_id = var.cloudflare_account_id
  email      = var.operator_email
}

resource "cloudflare_email_routing_address" "support_inbox" {
  provider   = cloudflare.email_routing
  account_id = var.cloudflare_account_id
  email      = "help@fortropp.helpscoutapp.com"
}

locals {
  email_forwards = {
    nathan  = cloudflare_email_routing_address.operator.email
    dmarc   = cloudflare_email_routing_address.operator.email
    support = cloudflare_email_routing_address.support_inbox.email
  }
}

resource "cloudflare_email_routing_rule" "operator" {
  for_each = local.email_forwards

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
      value = [each.value]
    }
  ]

  depends_on = [cloudflare_email_routing_dns.app]
}
