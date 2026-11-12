---
name: project-minimal-verified-worker
description: "User wants the Cloudflare worker's surface minimal and verified before production; prefers moving work to the final destination (R2, backend) over the worker"
metadata:
  node_type: memory
  type: project
  originSessionId: 064e4c62-d390-4763-b473-94b2c2fb6fc8
  modified: 2026-10-18T20:01:27.117Z
---

The user is not intimately familiar with Cloudflare, so before production they want the worker to do only what it strictly must (serving), with its behaviour verified (Rocq laws, corpus, coverage, hax pilot). Example: uploads moved from a Worker route back to R2 presigned PUTs with signed checksum + length (2026-10-18), shrinking the worker ~4%.

**Why:** fewer moving parts in an unfamiliar platform means less to worry about at launch; verification effort should target what remains.

**How to apply:** when a feature could live in the worker or elsewhere (R2, backend), default to elsewhere and ask; treat Rust↔Rocq coupling work (todo Misc) as pre-launch priority. Test-stack limitations (e.g. Miniflare not checking checksums) are never a reason to pick the weaker design; pin such guarantees against the real platform instead. Related: [[feedback-monadic-meaning]], [[project-worker-review-handoff]]
