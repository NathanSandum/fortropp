---
name: feedback-monadic-meaning
description: "What the user means by \"monadic\" for the worker flow; Rust stays idiomatic and plain, overlapping the Rocq model"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 064e4c62-d390-4763-b473-94b2c2fb6fc8
  modified: 2026-10-17T01:07:34.797Z
---

"Monadic" means: an abstract representation of the intended semantics that can be reified (interpreted) into the real behaviour. It does not mean monadic syntax.

**Why:** the user wants the sweet spot of verifiable and idiomatic code. Rust and the Rocq model should overlap structurally; monadic syntax (do-notation, `async` used as do-notation) would break that overlap. New crates (e.g. do-notation crates) are not refused, just not ideal.

**How to apply:** prefer plain enums, matches and data (effects as data, no async-for-syntax); propose a crate only with a clear payoff. Put the abstraction in the Rocq spec: one handler-independent program that every interpreter reifies, rather than a semantics written by hand against one store. Related: [[project-worker-review-handoff]]
