---
name: feedback-users-first-priority
description: "Launch priority — real functionality first, hardening last; defer protections that only matter once there are users"
metadata:
  node_type: memory
  type: feedback
  originSessionId: bc66f069-4c27-4ced-b5be-dcf8021a2216
  modified: 2026-11-10T00:24:42.297Z
---

Getting users is the ultimate blocker. Build real functionality first and hardening last; anything that only bites once there are users or scale (flood protection, speed-ups, polish) goes post-launch.

**Why:** the user, 2026-11-09 docs tidy: "We won't likely have issues until we have users, so this should wait until then, cause getting users is the ultimate blocker"; also "I'd mostly prefer to get the real functionality done first".

**How to apply:** when sorting work into launch vs post-launch, ask whether it matters before real users arrive. Advertised features (CDN, analytics existing at all) and things that break ordinary use (Safari video needing range requests, the shared KV quota) stay launch. Links: [[project-every-resource-capped]], [[project-paid-plans-at-launch]].
