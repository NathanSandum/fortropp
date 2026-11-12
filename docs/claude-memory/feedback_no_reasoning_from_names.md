---
name: feedback-no-reasoning-from-names
description: "Never base a design argument on what a page, route or component's name suggests; only on what it actually does today"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 7b9d6c82-cc6c-4ddb-a759-9a1f0c6a3776
  modified: 2026-11-09T00:35:05.214Z
---

Don't justify a design by what a page or route "is for" judging by its name. On 2026-11-08 I cited `/dashboard/` (a blank page whose purpose is undecided and which can't be reached once left) as a reason for a URL design; the user called it "making shit up just by name".

**Why:** an unbuilt or placeholder page teaches nothing; reasoning from its name invents product facts the user hasn't decided.

**How to apply:** before leaning on a page or component in an argument, check what it actually renders and how it's reached; if its purpose is undecided, leave it out of the reasoning. Related: [[feedback-discuss-product-decisions]], [[feedback-derive-from-failure-modes]].
