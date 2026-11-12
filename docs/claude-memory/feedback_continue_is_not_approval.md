---
name: feedback-continue-is-not-approval
description: "a bare \"continue\" (or a repeated message) is never approval for outward actions like releases, tags, deploys or pushes"
metadata:
  node_type: memory
  type: feedback
  originSessionId: e38e9579-8876-4e06-8048-11567f7460be
  modified: 2026-10-27T21:26:13.483Z
---

A short "continue"/"Continiue" or a message the user already sent can be their client resending after a dropped connection. Never read it as approval for an outward-facing or hard-to-reverse step (`just release`, tag pushes, deploys, deletions).

**Why:** 2026-10-27 a resent "Continue" after an internet outage was taken as approval and `just release` pushed tag `v2026.10.27.3` and started a CI build unasked.

**How to apply:** for those steps, ask explicitly and wait for an unambiguous yes that names or clearly answers the action; on "continue", finish only local, reversible work and ask about the rest. Related: [[feedback-weighty-actions-confirm]].
