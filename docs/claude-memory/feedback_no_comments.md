---
name: feedback-no-comments
description: User removes explanatory comments from code/SQL I write; keep code comment-free unless truly non-obvious
metadata:
  node_type: memory
  type: feedback
  originSessionId: c5a8d5e3-3e63-47bd-928f-859410e7ec19
  modified: 2026-10-03T02:40:45.710Z
---

User asked to strip explanatory comments from a migration I wrote (2026-10-02, fortropp `0001_init.sql`).

**Why:** codebase has very low comment density; rationale lives in todo_list.md / conversation, not inline.
**How to apply:** don't add explanatory comments to SQL or Rust in fortropp; at most a terse one where code would mislead without it.
