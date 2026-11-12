---
name: feedback-toml-not-json
description: "Config and metadata files we author are TOML, never JSON"
metadata:
  node_type: memory
  type: feedback
  originSessionId: acab82c6-b652-4b96-a4c8-07a31066ce4e
  modified: 2026-10-30T19:25:33.068Z
---

Hand-written config/metadata files in fortropp are TOML, never JSON (user, 2026-10-30: "100% in a million years not json"; JSON is a web-development habit, not a good config format).

**Why:** the user finds JSON a poor format for anything people write or read; fortropp already standardises on TOML (`fortropp.toml`, Cargo).

**How to apply:** propose TOML for new config or shared metadata files (e.g. the root `metadata.toml` for coordinated values like the site name); JSON stays only where an external API or tool dictates it (Postmark/Cloudflare payloads, package.json). Related: [[feedback-discuss-product-decisions]].
