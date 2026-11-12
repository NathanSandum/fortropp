---
name: project-fortropp-settings-architecture
description: "fortropp's WebsiteSettings.public field is moving from a DB bool to being sourced/mutated through an abstracted per-S3-backend layer, backed by Cloudflare R2"
metadata: 
  node_type: memory
  type: project
  originSessionId: b31807ac-28bf-4786-8787-96b4dc084542
  modified: 2026-09-13T22:59:25.110Z
---

fortropp's blob storage is Cloudflare R2, accessed via `aws_sdk_s3` pointed at R2's S3-compatible endpoint (`backend/src/blob_storage.rs`). R2's S3-compat surface does NOT include `GetBucketPolicy`/`PutBucketPolicy`. Public access on R2 is a Cloudflare-account-level concept instead — either the managed `r2.dev` subdomain toggle or a custom domain attached to the bucket — controlled via Cloudflare's own API (`/accounts/{id}/r2/buckets/{name}/domains/...`), which needs a separate Cloudflare API token, not the S3 access-key creds already in `Config`.

Current state (as of 2026-09-11, pre-change):
- `WebsiteSettings { public: bool }` in `backend/src/models/website.rs`, flattened into the `websites` table (`public BOOLEAN NOT NULL DEFAULT false`, `backend/migrations/0001_init.sql`).
- `Website::update` / `Website::find_settings` (website.rs) do plain DB read/write.
- `backend/src/api/websites.rs` — `PATCH /websites/{id}` handler wired to `Website::update`.
- Frontend: `frontend/src/lib/components/custom/Dashboard/WebsiteSettingsController.svelte.ts` (`setPublic`/`save`) calling `Websites.svelte.ts`'s `updateSettings`. Not wired to a real toggle yet.
- No bucket-policy or Cloudflare-API code exists yet in the repo — this is greenfield.

Decided direction (per [[feedback_external_source_of_truth]]):
- DB stops being the source of truth for `public`; remove the DB column/dead code once the new path lands.
- Build an `ExternalSettings`-style anti-corruption layer: a trait/tool abstracting "get/set public-ness of a bucket" so the concrete backend (Cloudflare R2 today) is swappable behind it, rather than hardcoding Cloudflare API calls into the settings/website code paths.
- Implement the CloudflareR2 backend for it first (talking to Cloudflare's account API for the r2.dev/custom-domain toggle, not the S3 SDK).
- Rate limiting on the settings-update endpoint is a stated follow-up (abuse/DDoS concern on the external call), explicitly not treated as an architecture blocker.

**Why:** user wants `WebsiteSettings` to keep growing fields that each dovetail to their own true owner (some may stay DB-backed, some external) rather than one blanket persistence strategy.

**How to apply:** when implementing, don't reach for `aws_sdk_s3` bucket-policy calls (unsupported by R2) — design the abstraction around Cloudflare's account API for the R2 backend, keep it generic enough that a future non-R2 S3 backend could implement the same trait differently.

**2026-09-13 refinement — split at the struct/UI level, not just persistence:**
User is proposing (not yet built) a further split: `public` moves fully OUT of `WebsiteSettings`/"Settings" and into a separate "control panel" concept — control panel becomes the owner of externally-backed, live-mutable state (starting with `public`/R2 toggle). Meanwhile "Settings" narrows to mean the stored/derived representation of a static-hosting config file (`fortropp.toml`) — import and routing config, analogous to `netlify.toml`/`vercel.json`. So the split isn't just "which backend stores this field" but "which subsystem (toml-derived Settings vs. control panel) even owns this field conceptually." Also floated: reading competitor tomls as a behavioral benchmark — early idea, not scoped.
Still exploratory as of this date — no toml schema committed yet.

**2026-09-13 — first step landed:** `WebsiteSettings` renamed to `WebsiteControls` end-to-end (backend struct `backend/src/models/website.rs`, `Website.settings` field → `Website.controls`, API JSON body/response key, frontend type/store in `Websites.svelte.ts`, controller/actions components renamed `WebsiteControlsController`/`WebsiteControlsActions`). The `public` toggle UI moved from the Settings page (`dashboard/(website)/settings/`) to the pre-existing empty Control Panel route (`dashboard/control-panel/+page.svelte`, already in the sidebar nav as "Control Panel"). Danger Zone (delete website) stayed on the Settings page per explicit user choice — it is not part of this externally-owned-controls split, just historically colocated. Activity-log event renamed too: `settings_updated_event` → `controls_updated_event` in `backend/src/values/website_name.rs`, message text `"Updated 'X' settings"` → `"Updated 'X' controls"`. e2e test `website_settings.e2e.ts` renamed to `website_control_panel.e2e.ts` and updated to match. Settings page is now otherwise empty, reserved for the future toml-derived config.
