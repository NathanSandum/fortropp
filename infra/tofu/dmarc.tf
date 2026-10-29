resource "cloudflare_dns_record" "dmarc" {
  zone_id = data.cloudflare_zone.app.zone_id
  name    = "_dmarc.fortropp.dev"
  type    = "TXT"
  content = "v=DMARC1; p=none; rua=mailto:dmarc@fortropp.dev"
  ttl     = 1
}
