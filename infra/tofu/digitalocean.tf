locals {
  region = "tor1"
}

resource "digitalocean_vpc" "production" {
  name   = "fortropp-prod"
  region = local.region
}

resource "digitalocean_tag" "backend" {
  name = "fortropp-backend"
}

resource "digitalocean_ssh_key" "operator" {
  name       = "fortropp"
  public_key = var.ssh_public_key
}

resource "tls_private_key" "deploy" {
  algorithm = "ED25519"
}

resource "digitalocean_ssh_key" "deploy" {
  name       = "fortropp-deploy"
  public_key = tls_private_key.deploy.public_key_openssh
}

resource "digitalocean_droplet" "backend" {
  name       = "fortropp-backend"
  region     = local.region
  size       = "s-1vcpu-2gb"
  image      = "ubuntu-24-04-x64"
  vpc_uuid   = digitalocean_vpc.production.id
  ipv6       = true
  monitoring = true
  ssh_keys   = [digitalocean_ssh_key.operator.fingerprint, digitalocean_ssh_key.deploy.fingerprint]
  tags       = [digitalocean_tag.backend.id]
  user_data  = file("${path.module}/cloud-init.yaml")
}

data "cloudflare_ip_ranges" "cloudflare" {}

locals {
  cloudflare_ranges = concat(
    data.cloudflare_ip_ranges.cloudflare.ipv4_cidrs,
    data.cloudflare_ip_ranges.cloudflare.ipv6_cidrs,
  )
}

resource "digitalocean_firewall" "backend" {
  name = "fortropp-backend"
  tags = [digitalocean_tag.backend.id]

  inbound_rule {
    protocol         = "tcp"
    port_range       = "22"
    source_addresses = ["0.0.0.0/0", "::/0"]
  }

  inbound_rule {
    protocol         = "tcp"
    port_range       = "80"
    source_addresses = ["0.0.0.0/0", "::/0"]
  }

  inbound_rule {
    protocol         = "tcp"
    port_range       = "443"
    source_addresses = local.cloudflare_ranges
  }

  outbound_rule {
    protocol              = "tcp"
    port_range            = "1-65535"
    destination_addresses = ["0.0.0.0/0", "::/0"]
  }

  outbound_rule {
    protocol              = "udp"
    port_range            = "1-65535"
    destination_addresses = ["0.0.0.0/0", "::/0"]
  }

  outbound_rule {
    protocol              = "icmp"
    destination_addresses = ["0.0.0.0/0", "::/0"]
  }
}

resource "digitalocean_database_cluster" "postgres" {
  name                 = "fortropp-postgres"
  engine               = "pg"
  version              = "18"
  size                 = "db-s-1vcpu-1gb"
  region               = local.region
  node_count           = 1
  private_network_uuid = digitalocean_vpc.production.id
}

resource "digitalocean_database_db" "fortropp" {
  cluster_id = digitalocean_database_cluster.postgres.id
  name       = "fortropp"
}

resource "digitalocean_database_user" "fortropp" {
  cluster_id = digitalocean_database_cluster.postgres.id
  name       = "fortropp"

  settings {}
}

resource "digitalocean_database_firewall" "postgres" {
  cluster_id = digitalocean_database_cluster.postgres.id

  rule {
    type  = "tag"
    value = digitalocean_tag.backend.id
  }
}

resource "digitalocean_project" "fortropp" {
  name        = "fortropp"
  purpose     = "Web Application"
  environment = "Production"
  resources = [
    digitalocean_droplet.backend.urn,
    digitalocean_database_cluster.postgres.urn,
  ]
}
