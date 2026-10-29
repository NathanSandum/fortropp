resource "cloudflare_dns_record" "postmark_dkim" {
  zone_id = data.cloudflare_zone.app.zone_id
  name    = "20261029021601pm._domainkey.fortropp.dev"
  type    = "TXT"
  content = "k=rsa;p=MIGfMA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQCqn/QkeHYd3oSOW1fQqNxVHjpkkyWs4qiE6In+Km6TNK8YF9K/9kVG7mnbzmUXKjgOyoSkgwpvM1OIYylIAW8XikRetICWzOSiLHmohWd7zXN+3Fa2XHOLmHDg3FlvwnWEX5wj4+2LGreKTjELfaQP+hSQBOf3xAi78t8GL0KnkwIDAQAB"
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
