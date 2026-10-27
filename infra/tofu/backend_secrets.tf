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

output "database_url" {
  value = format(
    "postgres://%s:%s@%s:%d/%s?sslmode=require",
    digitalocean_database_user.fortropp.name,
    urlencode(digitalocean_database_user.fortropp.password),
    digitalocean_database_cluster.postgres.private_host,
    digitalocean_database_cluster.postgres.port,
    digitalocean_database_db.fortropp.name,
  )
  sensitive = true
}

output "s3_endpoint" {
  value = "https://${var.cloudflare_account_id}.r2.cloudflarestorage.com"
}

output "kv_api_url" {
  value = "https://api.cloudflare.com/client/v4/accounts/${var.cloudflare_account_id}"
}

output "kv_namespace_id" {
  value = cloudflare_workers_kv_namespace.site_routes.id
}

output "cloudflare_hostnames_zone_id" {
  value = data.cloudflare_zone.user_content.zone_id
}

output "origin_certificate" {
  value = cloudflare_origin_ca_certificate.app.certificate
}

output "origin_private_key" {
  value     = tls_private_key.app_origin.private_key_pem
  sensitive = true
}
