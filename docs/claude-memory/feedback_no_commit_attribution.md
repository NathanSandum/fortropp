---
name: feedback-no-commit-attribution
description: Never add Co-Authored-By / Claude attribution lines to commits (or PR descriptions) in fortropp
metadata:
  node_type: memory
  type: feedback
  originSessionId: c5a8d5e3-3e63-47bd-928f-859410e7ec19
  modified: 2026-10-04T06:20:16.753Z
---

User rejected commits containing a `Co-Authored-By: Claude …` trailer (2026-10-04): "don't cite yourself".

**Why:** user's own preference for their repos' history; overrides the harness's default attribution reminder.
**How to apply:** commit messages (and PR bodies) in fortropp and its submodules get no Claude/AI attribution trailers or footers.
