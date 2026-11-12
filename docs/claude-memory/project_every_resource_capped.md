---
name: project-every-resource-capped
description: Product principle — every user-creatable resource in fortropp must have a cap (only the leniency is negotiable)
metadata:
  node_type: memory
  type: project
  originSessionId: c5a8d5e3-3e63-47bd-928f-859410e7ec19
  modified: 2026-10-03T18:23:19.410Z
---

User, 2026-10-03: "Literally every resource should have some kind of a cap."

**Why:** abuse/cost control for a solo-run service on metered infra (e.g. Cloudflare for SaaS bills per custom hostname); uncapped resources are a liability.
**How to apply:** when designing any new resource (domains, websites, files, pending claims, tokens, etc.), include a cap in the design up front and ask the user only about how lenient it should be (possibly per plan tier: Starter/Growth/Scale). Related: [[feedback-discuss-product-decisions]].
