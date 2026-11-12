---
name: project-setting-write-invariant
description: fortropp frontend rule — settings writes are SettingWrite values run only via SettingWriteButton or confirmThen
metadata:
  node_type: memory
  type: project
  originSessionId: d9dd79b7-f6e3-41bc-92bc-b8afa0c2a636
  modified: 2026-10-10T17:35:44.745Z
---

Every frontend **settings** write (preview link, Public toggle, domain actions, fortropp.toml save, publish) is a `SettingWrite<T>` value with a private run function. Only two interpreters may execute it: `SettingWriteButton` (direct; the button label comes from the write itself) and `confirmThen(write, next)` (side effect; always shows the write's confirm modal). Weighty writes set `confirmEvenWhenDirect`. File uploads/deletes are draft content and are NOT settings writes. Boundary = per-website settings + publish only: delete website, API tokens/agents, create website and auth flows are NOT SettingWrites — the user sees those as different kinds of effect that would each get their own distinct type if they ever need an invariant, never folded into SettingWrite.

Shape: a pointed functor today (`SettingWrite.pure` = no change → no modal, no request; `map` with pure callbacks; functor laws covered by unit tests). If combining several writes into one confirm is ever needed, add applicative `zip`/`ap` (combining no-ops stays a no-op; one modal lists every real change). **Never add `flatMap`**: a write that depends on another write's result can't be confirmed honestly up front.

**Why:** the user wants an enforced invariant that a settings change made as a side effect of another action (e.g. Preview turning a preview link back on) can never happen without a confirm modal (decided 2026-10-10).

**How to apply:** never add a store method that performs a settings write directly; expose a `SettingWrite` instead and render it with one of the two interpreters. Related: [[feedback-weighty-actions-confirm]], [[feedback-discuss-product-decisions]].
