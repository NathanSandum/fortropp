---
name: project-handoff
description: "Read docs/handoff.md in the main repo at session start, its \"Start here\" section first; as of 2026-11-09 production is up with one-record root domains, and the next work is billing's design discussion"
metadata:
  node_type: memory
  type: project
  originSessionId: bc66f069-4c27-4ced-b5be-dcf8021a2216
  modified: 2026-11-15T17:36:30.970Z
---

`/Users/nsandum/Projects/fortropp/docs/handoff.md` holds the current state, the production plan's progress, how to verify, gotchas and the tools on this machine; every decision with its reasons lives in `production_notes.md`; `todo_list.md` is the launch list (reviewed with the user 2026-11-09: launch vs post-launch sorted, a Launch prep section), `post_launch_todo_list.md` the rest.

As of 2026-11-09: `v2026.11.09.2` is deployed (everything from that day, main `b1607bb`); one-record root domains work on real Cloudflare (`voltery.ca` live with no warning window). Done that day: the `unwrap_or_default` ban, case-insensitive emails, the routing config rename (MCP tools `read_routing_config`/`write_routing_config`), e2e accounts and websites created over the API (parallel phase 9.8 → ~5 min), worker memory traced to Miniflare's local emulation (production flat at ~4 MB). 2026-11-10: billing design done, recorded in `todo_list.md`'s "Billing, page views and limits" section (handoff "Start here" has the summary); next: build it, starting from Stripe's own test tools. 2026-11-10: billing step 1 (plans and caps) committed locally on `billing` branches; step 2 (Stripe) next. 2026-11-14: billing mostly built on `billing` branches in all four repos (worker too), not pushed: Stripe invoicing, plan switching, shutdown with the worker's per-website toggle, paying from the page, billing history. Next: billing emails, then card change, drawer and waiver wording, deletion settle-up, then page views and analytics (handoff "Start here" has the list). 2026-11-15: billing emails done and committed locally (backend `693a696`, frontend `102eee5`, main `fe8edb5`), all tests and the 8 sandbox tests passing; next: changing the card, then the database review after billing.

**Why:** carries state between sessions.

**How to apply:** read it before starting work, then `production_notes.md` and `todo_list.md` where relevant. Replace it (and this memory) when it goes stale. Related: [[feedback-discuss-product-decisions]], [[feedback-users-first-priority]], [[project-minimal-verified-worker]].
