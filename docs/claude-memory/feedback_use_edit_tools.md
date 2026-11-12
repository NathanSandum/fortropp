---
name: feedback-use-edit-tools
description: "Use Edit/Write for file changes, never python/sed/heredoc scripts via Bash"
metadata:
  node_type: memory
  type: feedback
  originSessionId: c5a8d5e3-3e63-47bd-928f-859410e7ec19
  modified: 2026-10-03T03:03:02.075Z
---

User rejected a python-in-Bash file rewrite (2026-10-02): "why is this a bash command, and not a normal file edit?"

**Why:** Bash-scripted edits are opaque to review and bypass the edit tooling's diff view.
**How to apply:** all source/config/doc edits go through Edit (or Write for new/full rewrites), even multi-spot edits — just issue several Edit calls.
