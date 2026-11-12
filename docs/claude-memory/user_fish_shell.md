---
name: user-fish-shell
description: "user's interactive shell is fish; commands handed to them must be fish-compatible or wrapped in a bash script"
metadata:
  node_type: memory
  type: user
  originSessionId: e38e9579-8876-4e06-8048-11567f7460be
  modified: 2026-10-27T04:30:44.436Z
---

The user's terminal runs fish (Claude's own Bash tool runs zsh). Commands given to the user to paste must work in fish: no `( … )` subshells, `set -a`, `export X=`, `$(…)` (fish ≥3.4 accepts `$(…)`, but avoid bash-only syntax generally). For anything bash-shaped, write a script to the scratchpad and hand them `bash <path>`.

**How to apply:** especially for secret-handling steps the user runs themselves (their `op` session is manual sign-in, see [[feedback-never-print-credential-files]]).
