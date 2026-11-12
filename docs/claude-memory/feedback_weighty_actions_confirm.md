---
name: feedback-weighty-actions-confirm
description: "Consequential actions (e.g. publishing a site live) get a confirm dialog, not an instant one-click action"
metadata:
  node_type: memory
  type: feedback
  originSessionId: d9dd79b7-f6e3-41bc-92bc-b8afa0c2a636
  modified: 2026-10-09T21:35:52.170Z
---

Weighty, user-visible actions like Publish should open a confirm dialog (listing what will happen) rather than fire instantly with a toast.

**Why:** The user said publishing "feels weighty" and a confirm dialog is the only visual way to validate that weight; an instant update "may almost feel jarring". They overrode my "instant + toast" recommendation for this reason (2026-10-09).

**How to apply:** When proposing UX for actions that change what visitors see or are hard to undo (publish, rollback, going public), default to a confirm dialog, like `RemoveDomainDialog`/`PublishDialog`. Don't recommend one-click instant flows for these. Related: [[feedback-discuss-product-decisions]].
