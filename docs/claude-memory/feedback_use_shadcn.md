---
name: feedback-use-shadcn
description: Frontend UI must use the shadcn-svelte components ($lib/components/ui/*) for every control — no raw <button>/<input> etc.
metadata:
  node_type: memory
  type: feedback
  originSessionId: c5a8d5e3-3e63-47bd-928f-859410e7ec19
  modified: 2026-10-03T20:02:10.788Z
---

User, 2026-10-03, rejecting a raw `<button>` I wrote: "make sure the shadcn elements are used everywhere to ensure consistent styling."

**Why:** consistent styling across the dashboard; raw elements drift from the design system.
**How to apply:** in fortropp's frontend, reach for `$lib/components/ui/*` (Button incl. `variant="link"`, Input, Checkbox, Table, Dialog, DropdownMenu, Field, Badge, …) for every interactive or structural element; only use plain elements for text (p, span, headings) where no shadcn equivalent exists.
