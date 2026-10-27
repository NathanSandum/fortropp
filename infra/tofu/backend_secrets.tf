resource "random_bytes" "session_key" {
  length = 64
}

resource "random_bytes" "domain_verification_key" {
  length = 32
}

output "session_key" {
  value     = random_bytes.session_key.hex
  sensitive = true
}

output "domain_verification_key" {
  value     = random_bytes.domain_verification_key.hex
  sensitive = true
}

output "origin_certificate" {
  value = cloudflare_origin_ca_certificate.app.certificate
}

output "origin_private_key" {
  value     = tls_private_key.app_origin.private_key_pem
  sensitive = true
}
