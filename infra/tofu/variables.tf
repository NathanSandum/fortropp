variable "cloudflare_account_id" {
  type = string
}

variable "state_passphrase" {
  type      = string
  sensitive = true
}

variable "ssh_public_key" {
  type = string
}

variable "cloudflare_email_routing_token" {
  type      = string
  sensitive = true
}

variable "operator_email" {
  type      = string
  sensitive = true
}
