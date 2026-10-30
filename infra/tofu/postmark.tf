resource "cloudflare_dns_record" "postmark_dkim" {
  zone_id = data.cloudflare_zone.app.zone_id
  name    = "20261029032146pm._domainkey.fortropp.dev"
  type    = "TXT"
  content = "k=rsa;p=MIGfMA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQCyIj5mmDUEJn3+5eHVFngmX0hzCZrLhGYoYcvTlpLqoX4FCI3RWRfHco0uBt/0Ey3rDn/Xyi6ty1vkEXE6CPdAyy7ujsP74bMvis0WNo0mz6eQjdr1Xpz2ewG9piB52hAN3yWo2AyDdVrYcxQtwtAAPDdrjF7hpF0INNsOo8BwFQIDAQAB"
  ttl     = 1
}

resource "cloudflare_dns_record" "postmark_return_path" {
  zone_id = data.cloudflare_zone.app.zone_id
  name    = "pm-bounces.fortropp.dev"
  type    = "CNAME"
  content = "pm.mtasv.net"
  proxied = false
  ttl     = 1
}
