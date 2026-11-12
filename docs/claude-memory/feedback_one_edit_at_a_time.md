---
name: feedback-one-edit-at-a-time
description: "Send file edits one per turn step, never as a parallel batch, so a rejection stops the rest"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 42149cc9-0d07-438b-8ae8-e1d00931fcd5
  modified: 2026-10-24T16:20:29.487Z
---

Make edits sequentially, one tool call at a time, especially for prose/docs the user reviews closely. Never batch several Edit calls in parallel.

**Why:** When the user rejects one edit in a parallel batch, the remaining ones keep prompting anyway; the user found this infuriating (2026-10-07, fortropp.toml docs update).

**How to apply:** Issue one Edit, wait for the result, then the next. If an edit is rejected, stop and address the feedback before proposing more. Same for commits: submodule commits, then the main-repo bump, strictly in sequence (2026-10-24 a parallel batch pushed a main-repo bump before the frontend commit it relied on had succeeded). Related: [[feedback-discuss-product-decisions]].
