---
name: destructive-steps-last
description: "In multi-step plans, schedule renames/deletions/irreversible steps last, after the substantive work"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 84b36859-9a85-4782-9415-77412f628d58
  modified: 2026-10-11T19:04:53.184Z
---

Order plans so destructive or rename steps (repo renames, removing submodules/routes, deleting code paths) come as late as possible.

**Why:** the plan may change as we develop a feel for the work; renames aren't the important part and are costly to undo.

**How to apply:** build the new thing alongside the old first; rename/remove at the end, after checking in. Related: [[confirm-weighty-actions]].
