---
name: project-metadata-vs-config
description: metadata.toml holds facts about the product (name); real config flows OpenTofu → backend → frontend
metadata:
  node_type: memory
  type: project
  originSessionId: acab82c6-b652-4b96-a4c8-07a31066ce4e
  modified: 2026-10-31T00:45:55.932Z
---

Root `metadata.toml` is for facts about the product that hold regardless of deployment, like the site name: renaming the product changes it, nothing else does (user, 2026-10-30). Real configuration that may change on occasion (e.g. the Turnstile sitekey, produced by OpenTofu) does not go there: it flows OpenTofu output → `just prod-secrets` → backend config → frontend at runtime (e.g. a backend config endpoint), the same pipe as other OpenTofu-produced values.

**Why:** a single source of truth per value; config values follow their producer (OpenTofu) automatically instead of being copied into the repo.

**How to apply:** before adding a value to `metadata.toml`, ask whether it is a fact about the product or deployment config; config goes through the backend. Related: [[feedback-external-source-of-truth]], [[feedback-toml-not-json]].
