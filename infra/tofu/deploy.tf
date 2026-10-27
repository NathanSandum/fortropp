output "deploy_host" {
  value = digitalocean_droplet.backend.ipv4_address
}

output "deploy_private_key" {
  value     = tls_private_key.deploy.private_key_openssh
  sensitive = true
}
