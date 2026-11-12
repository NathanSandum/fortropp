---
name: feedback-typed-json
description: No ad-hoc json! in production code; define serializable types so request and response shapes are auditable
metadata:
  node_type: memory
  type: feedback
  originSessionId: 8133b634-d7e8-4f9d-959f-826045d58546
  modified: 2026-11-03T04:22:06.723Z
---

Define a serializable type for any JSON body in production code, not `serde_json::json!`.

**Why:** the user wants types to be auditable, and ad-hoc JSON obscures them (2026-11-02).

**How to apply:** new code uses typed bodies. Tests keep literal JSON, since that is what they assert. The fakes in `infra/test-services` may be typed someday, which would harden the model of the external world. The remaining production uses are a Misc item in `todo_list.md`.
