---
name: feedback-external-source-of-truth
description: "Settings owned by an external system must query/mutate that system directly, not cache a shadow value in Postgres"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: b31807ac-28bf-4786-8787-96b4dc084542
  modified: 2026-09-11T21:30:04.717Z
---

When a `WebsiteSettings` field's real behavior is determined by an external system (e.g. R2/S3 bucket public-access config), do not store a duplicate value for it in the DB. Query and mutate the external system directly for that field. DB-as-cache is explicitly rejected here even though it removes a race/drift window on our side.

**Why:** if the external system's state changes independent of us (someone edits it directly), the DB value becomes wrong while still being reported as truth — user sees one behavior, we tell them another. That confusion is worse than any race. Also: reasoning like "cache to avoid an extra API call" doesn't hold when the call is cheap/metered by volume (e.g. Cloudflare R2 calls billed per-million) — cost isn't the architecture concern; abuse/DDoS on the endpoint is a rate-limiting concern, not a reason to introduce a cache layer.

**How to apply:** `WebsiteSettings` is expected to grow more fields over time, each potentially owned by a different subsystem. Don't design one blanket persistence strategy for the whole object — each field dovetails to its own actual source of truth. See [[project_fortropp_settings_architecture]] for the concrete shape being built (per-backend anti-corruption layer, `ExternalSettings`-style abstraction) for the `public` field specifically.
