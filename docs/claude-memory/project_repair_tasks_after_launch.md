---
name: project-repair-tasks-after-launch
description: skipping repair tasks for multi-release rollbacks is pre-launch only; after launch such releases need one
metadata:
  node_type: memory
  type: project
  originSessionId: e99f7a3b-03a5-4105-9efe-ea09a44c6ef1
  modified: 2026-11-04T17:20:27.285Z
---

Release C of the single preview link dropped its repair task (rebuilding `preview:` KV keys from Postgres after a rollback past B), decided 2026-11-04.

**Why:** the user accepted this "for this run" because there are no real users yet; a bad rollback only leaves a preview link showing the wrong content until someone moves it. They said that after launch such a task is "probably a requirement".

**How to apply:** when planning expand-migrate-contract releases after launch, include a repair task for derived state (KV, caches) that a rollback can leave out of step, removed in a later contract step. Before launch, skipping it is fine if the failure is named and tolerable.
