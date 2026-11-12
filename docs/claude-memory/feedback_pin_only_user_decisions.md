---
name: feedback-pin-only-user-decisions
description: "Docs record only what the user decided; never write an LLM's suggestion as a constraint, and don't cite old doc \"constraints\" as user decisions"
metadata:
  node_type: memory
  type: feedback
  originSessionId: f90bca9b-7cd0-4c54-9b89-a371474d883b
  modified: 2026-11-10T15:44:30.708Z
---

Only write a constraint or decision into the repo docs (todo_list, todo_archive, production_notes, handoff) when the user said it. An assistant's suggestion goes in as an open option, if at all.

**Why:** 2026-11-10: "a visit is not a request" sat in `todo_archive.md` as a constraint; it came from an LLM, not the user, and was then quoted back to the user as settled. The user had to keep correcting it.

**How to apply:** before presenting a doc line as "already decided", check it reads as the user's call; if unsure of its source, present it as open. When recording, attribute decisions to the user explicitly. Related: [[feedback-todo-declarative]], [[feedback-discuss-product-decisions]].
