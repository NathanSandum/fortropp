---
name: project-button-taxonomy
description: "fortropp frontend buttons only via custom/Button roles (Primary, Secondary, Danger, Quiet, Icon, Inline, Cta), never ui/button directly"
metadata:
  node_type: memory
  type: project
  originSessionId: d9dd79b7-f6e3-41bc-92bc-b8afa0c2a636
  modified: 2026-10-10T17:51:29.519Z
---

All frontend buttons use `$lib/components/custom/Button` roles, never `$lib/components/ui/button` directly (lint-enforced):
- `Button.Primary` (shadcn default, sm): the one main action of a view/dialog — Publish, Save, Make public, Add Website.
- `Button.Secondary` (outline, sm): supporting/standalone non-destructive — Edit, Cancel, Create link, Preview.
- `Button.Danger` (destructive, sm): confirming something irreversible — Remove, Turn off link, Delete.
- `Button.Quiet` (ghost text, sm): disclosure/toggle triggers — "3 unpublished changes ▾".
- `Button.Icon` (ghost, icon-sm, required label): icon-only row actions, menu triggers, Copy.
- `Button.Inline` (link): text links inside sentences — Preview first, View all backups.
- `Button.Cta` (default, lg): marketing pages only.
Dashboard uses a single size (sm).

**Why:** the user noticed visually inconsistent buttons (e.g. Create link outline vs Turn off ghost) chosen by feel, and asked for an internal taxonomy so we stop straying (approved 2026-10-10, with Quiet added).

**How to apply:** pick a role by what the button *does*, not how it should look; never pass raw shadcn variants. Interpreters like `SettingWriteButton` take a role, not a variant. Related: [[project-setting-write-invariant]], [[feedback-use-shadcn]].
