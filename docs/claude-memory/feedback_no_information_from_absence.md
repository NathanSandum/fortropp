---
name: feedback-no-information-from-absence
description: "Never turn missing data into a claim (e.g. no Stripe reason ⇒ \"your bank asked for approval\"); show only what the source said"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 78b09cf3-c3e6-44f8-a476-a4b661acb5be
  modified: 2026-11-15T15:46:30.836Z
---

When a source gives no information (a null Stripe failure reason, an absent field), user-facing text says nothing in its place: never infer a cause from the absence and present it as fact.

**Why:** 2026-11-15, proposing "Your bank asked you to approve this payment." for failed-charge emails with no Stripe reason; user: "We do not take no information and try to turn it into information."

**How to apply:** an optional sentence is left out when its data is missing. Related: [[feedback-no-manufactured-defaults]], [[feedback-no-invented-placeholders]].
