---
name: feedback-prelaunch-rollback-not-a-design-input
description: "Pre-launch, never shape a design around rollback compatibility (old binary vs new schema); the launch rebuild collapses migrations"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 7b9d6c82-cc6c-4ddb-a759-9a1f0c6a3776
  modified: 2026-11-09T17:33:54.818Z
---

Before launch, don't let rollback compatibility (the previous backend running against a new schema, the rollback rehearsal) drive design choices or keep being raised as a concern. Design for the post-launch end state; note launch-rebuild cleanups (e.g. `NOT NULL`) in the launch-rebuild todo item instead.

**Why:** 2026-11-09 the user said it "locks in our design into a concern that won't exist post-launch", after I kept bringing up rollback windows for `dns_label` and the root-class claims.

**How to apply:** pre-launch, pick the cleanest schema and mention a rollback gap at most once as a fact, not as a reason. After launch this flips: multi-release expand/contract applies. Related: [[project-repair-tasks-after-launch]], [[feedback-derive-from-failure-modes]].
