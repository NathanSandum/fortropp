resource "cloudflare_dns_record" "helpscout_dkim" {
  for_each = toset(["strong1", "strong2"])

  zone_id = data.cloudflare_zone.app.zone_id
  name    = "${each.key}._domainkey.fortropp.dev"
  type    = "CNAME"
  content = "${each.key}._domainkey.helpscout.net"
  proxied = false
  ttl     = 1
}
