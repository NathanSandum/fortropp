---
name: feedback-no-invented-placeholders
description: "Input placeholders name the field plainly (\"Key Name\", \"Version Name\"); never invented example content"
metadata:
  node_type: memory
  type: feedback
  originSessionId: fc2c38e4-5528-455b-8580-d61329f943c2
  modified: 2026-10-21T23:42:54.613Z
---

Placeholders in fortropp's frontend inputs are plain field names, e.g. "Key Name" (API token), "Version Name" (publish). Never invent example content like "New pricing page" or "Claude Code on my laptop".

**Why:** user (2026-10-21) called the publish field's "New pricing page" LLM slop: it sounds plausible in the abstract but fits almost no real edit. Too-specific examples help nobody.

**How to apply:** for new inputs use the field's name as the placeholder (format hints like `/from` or `Header-Name` are fine). The user's own placeholders (`pinky@acmelabs.com`, `bestblog.com`, `magnum-opus`) are deliberate; leave them.
