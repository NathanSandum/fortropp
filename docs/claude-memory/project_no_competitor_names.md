---
name: project-no-competitor-names
description: Keep Cloudflare and other competitors out of user-facing copy unless naming them is necessary
metadata:
  node_type: memory
  type: project
  originSessionId: 8133b634-d7e8-4f9d-959f-826045d58546
  modified: 2026-11-01T21:19:39.968Z
---

Don't name Cloudflare (or other competitors) in user-facing docs and copy; describe the behaviour instead ("network hops").

**Why:** the target audience is people building with agents who don't yet see the infrastructure they'll need. An LLM can make Cloudflare approachable enough that it competes for them, so the user would rather not advertise it (2026-11-01; extent still being decided, "unsure to what extent").

**How to apply:** default to neutral wording in docs, UI and error messages. Exceptions need a reason: the privacy policy's processor list, and telling a user which blocked host to unblock (`challenges.cloudflare.com` in `Turnstile.svelte`). Internal notes and code are unaffected.
