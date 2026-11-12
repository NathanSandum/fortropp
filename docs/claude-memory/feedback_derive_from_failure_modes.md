---
name: feedback-derive-from-failure-modes
description: "Justify each mechanism (timers, thresholds, broad sweeps) by a concrete failure mode; the user rejects heuristics standing in for real dependencies"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 0ccc27fa-d91c-4c88-8c50-11c33f0d9e12
  modified: 2026-10-26T00:16:29.386Z
---

Before proposing a periodic job, an age threshold, or a broad "refresh everything nearby" step, name the exact failure it handles and check whether a precise mechanism exists. Prefer state the system already has (status columns as a write-ahead log, idempotent status-guarded steps, derived data) over new bookkeeping.

**Why:** 2026-10-25, designing custom-domain registration, the user rejected in turn: a cron reconciler as the primary mechanism ("all events we need, we have access to"), an every-minute recovery job ("what creates the need?"), a per-attempt state table, a 5-minute in-flight threshold ("as fake a concept as it looks"), and route refreshes for whole websites instead of the actual redirect dependency. Each time the simpler, exact design was better. They also asked to keep re-deriving rather than defending the first design ("You're hung up on the same design still").

**How to apply:** when a design question comes up, list the concrete interruptions/events first, then the minimal mechanism per event; offer the precise version up front. Related: [[feedback-discuss-product-decisions]], [[project-minimal-verified-worker]].
