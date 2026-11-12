---
name: feedback-no-manufactured-defaults
description: "Never manufacture state from missing data (unwrap_or_default, Default on domain types); model absence with Option or an error"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 7b9d6c82-cc6c-4ddb-a759-9a1f0c6a3776
  modified: 2026-11-09T06:08:57.956Z
---

Don't fill missing data with defaults: no `unwrap_or_default`, no `Default` impls on domain types used to stand in for "absent", no `map_or_else(T::default, …)`. The user treats values as linear: absence is its own state (`Option`) or a failure (an error), never an empty value that looks like real data.

**Why:** 2026-11-09, I wrote `envelope.result.unwrap_or_default()` (Cloudflare success without a result became "nothing pending") and `Challenges::default()` on a cache miss; both hide malformed or missing data as legitimate empty state. The user plans a clippy ban (`disallowed_methods` on `Option/Result::unwrap_or_default`).

**How to apply:** empty accumulators a computation starts from (e.g. `FileSet::default()` before inserting) and library config constructors are fine; filling a gap in data received or looked up is not. Related: [[feedback-derive-from-failure-modes]], [[feedback-typed-json]].
