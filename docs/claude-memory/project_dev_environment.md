---
name: project-dev-environment
description: fortropp dev stack runs over HTTPS with a Caddy local CA; Claude Code trusts it and has the dev MCP server connected; repos are private on GitHub
metadata:
  node_type: memory
  type: project
  originSessionId: 42149cc9-0d07-438b-8ae8-e1d00931fcd5
  modified: 2026-11-12T21:08:22.257Z
---

Set up 2026-10-06/07:
- Dev stack serves https://localhost (Caddy `tls internal`); `just dev-ca` exports the root cert to `infra/dev-ca.crt` (gitignored). The user clicks through the browser warning rather than trusting the CA system-wide.
- Claude Code trusts it via `NODE_EXTRA_CA_CERTS` in the main repo's `.claude/settings.local.json`; after a Caddy volume wipe, re-run `just dev-ca`.
- MCP server `fortropp-dev` (https://localhost/api/mcp) is registered in local scope and signed in through OAuth, so Claude Code can call fortropp tools against dev.
- Vite (localhost:5173) proxies `/api` to the backend published on 127.0.0.1:8080. A Vite restart is needed when a new component directory is added, or Tailwind misses its classes.
- `just test` stops the dev stack; bring it back with `just dev-up`.
- Since 2026-10-24 the user's Mac has `/etc/resolver/test` → dev PowerDNS on 127.0.0.1:5300, so any `.test` name resolves in dev; Caddy serves `.test` hosts with on-demand internal certs (each new hostname is another browser warning to click through). Recipes: `just dev-domain`, `just dev-proof`.
- Repos are private on GitHub under NathanSandum: fortropp (main), fortropp-backend (`master`), fortropp-frontend, fortropp-worker (renamed from fortropp-dev-worker 2026-10-13; submodule path `worker/`, compose service `worker`); `.gitmodules` uses relative URLs. Push submodules before bumping the main repo.
- The worker's `site-config` crate is a path dependency of the backend (`../worker/site-config`); the backend image gets it through a compose `additional_contexts` entry.
- The planning docs (`todo_list.md`, `post_launch_todo_list.md`, `handoff.md`, `production_notes.md`, `todo_archive.md`) live in the main repo's `docs/`, tracked on `main` since 2026-11-12.

**Why:** these took a session to establish and aren't derivable from code alone.
**How to apply:** check here before debugging HTTPS, MCP or Vite proxy issues in dev; see [[feedback-one-edit-at-a-time]] for working style.
