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




variable "operator_email" {
  type      = string
  sensitive = true
}
