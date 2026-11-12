---
name: feedback-pr-vs-merge
description: "Small, already-verified changes merge straight to the default branch; PRs only for large changes that need their own review"
metadata:
  node_type: memory
  type: feedback
  originSessionId: fc2c38e4-5528-455b-8580-d61329f943c2
  modified: 2026-10-22T18:45:21.336Z
---

Small changes that `just test` already verified (e.g. dependency bumps) get committed, fast-forward merged into each repo's default branch (worker `main`, backend `master`, frontend `main`) and pushed, without a PR. Submodules first, then bump the main repo.

**Why:** user (2026-10-21): "PRs are good for when it's very large and is a task in itself to verify."

**How to apply:** open a PR only when reviewing the change is a task of its own (e.g. item 17's worker rewrite); still ask before committing. Related: [[feedback-no-commit-attribution]].

**Will change (decided 2026-10-22):** once the GitHub repos are load-bearing for production (CI deploys from version tags), main gets protected and every change goes through a PR with squash merge. Until that switch actually happens, the rule above stands. See `production_notes.md` §8.
