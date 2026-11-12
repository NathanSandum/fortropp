---
name: feedback-questions-are-not-instructions
description: "A question (\"what's the minimum…?\", \"is there a way…?\") gets an answer, never an action; don't stash, revert or change the working tree unasked"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 7b9d6c82-cc6c-4ddb-a759-9a1f0c6a3776
  modified: 2026-11-09T20:31:05.429Z
---

When the user asks a question, answer it and wait. Never act on it: no stashing, reverting, committing or reshaping the working tree because a question hinted at a direction.

**Why:** 2026-11-09 the user asked "What's the minimum amount to get this committable?" while low on session; I stashed the in-progress e2e account pool without asking. They then ran `just test` (15 minutes) to test that work, and it tested the stashed-away tree instead. "All I was doing was asking questions."

**How to apply:** if a question suggests an action that touches files or git state, describe the action and ask before doing it. This holds most of all for anything that changes what a running or upcoming test run sees. Related: [[feedback-continue-is-not-approval]], [[feedback-weighty-actions-confirm]].
