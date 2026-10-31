resource "cloudflare_ruleset" "rate_limit" {
  provider = cloudflare.waf
  zone_id  = data.cloudflare_zone.app.zone_id
  name     = "fortropp rate limits"
  kind     = "zone"
  phase    = "http_ratelimit"
  rules = [
    {
      ref         = "human_forms_and_authorize"
      description = "Sign-in, sign-up, reset and OAuth authorize per IP"
      expression  = "(http.request.uri.path in {\"/api/auth/sign_in\" \"/api/auth/sign_up/start\" \"/api/auth/reset_password/start\" \"/api/oauth/authorize\"})"
      action      = "block"
      ratelimit = {
        characteristics     = ["ip.src", "cf.colo.id"]
        period              = 10
        requests_per_period = 5
        mitigation_timeout  = 10
      }
    }
  ]
}
