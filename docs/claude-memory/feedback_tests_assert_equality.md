---
name: tests-assert-equality
description: "Unit tests assert the whole outcome equals an expected value, never that it contains/includes parts"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 84b36859-9a85-4782-9415-77412f628d58
  modified: 2026-10-14T18:36:27.676Z
---

Unit tests must say "the outcome IS X" (full equality of the complete result: every field, every header, the whole effect list), not "the outcome CONSISTS OF / contains X" (checking only a status, one header, `is_some`, `matches!`, `is_err`, `toMatchObject`).

**Why:** user (2026-10-14, item 17 review): equality guarantees nothing unexpected lives in the result; inclusion checks let extra fields, headers or effects slip through. Goes with their interest in verifiable, effects-as-data flows ([[destructive-steps-last]] review context).

**How to apply:** build the complete expected value with small helpers and `assert_eq!` it against the complete actual value (e.g. `(needs, reply)` tuples, full `Resolution` incl. headers, exact error variants). Strip nothing. HTTP-level e2e tests can't compare runtime-added headers (Date, CF-*), so there equality on the fields we control is the closest; say so when it applies.
