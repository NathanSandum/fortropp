---
name: project-viking-logo
description: "Viking logo (favicon + FinnTheHuman replacement) paused 2026-10-28 at variant CC; files in design/, user's design feedback so far"
metadata:
  node_type: memory
  type: project
  originSessionId: fe699485-7056-4410-b806-683569e2cc22
  modified: 2026-10-28T18:44:06.601Z
---

Goal: one Phosphor-style monochrome viking replaces the Svelte favicon (`frontend/src/lib/assets/favicon.svg`) and possibly the FinnTheHuman Profile icon. Paused 2026-10-28 because iterations stopped converging; user wants to come back later. Nothing wired into the app yet.

Committed in the frontend repo (bd1e24b, 2026-10-28): `frontend/design/viking-cc.svg` (current best, "CC"), `frontend/design/logo-lab/index.html` (live-edit lab holding CC + CA, CA = CC's smooth beard shape; serve with frontend's vite: `vite --port 5181` in that dir), `frontend/design/logo-lab/beard.mjs` (generates scalloped beard paths along a base cubic).

User's direction so far:
- Monochrome, Phosphor regular look (256 grid, round caps) — colourful versions were jarring.
- Horned helmet, egg face (round but tapering slowly, not "mumps"), bowl eyes (thin ∪, no top line — ambiguity between eyes and old spectacles, NOT an emotion), mustache that hands off face→beard, beard that droops to a rounded point with cloud puffs as texture on a defined shape.
- Thinner strokes for eyes (6) and beard/mustache (12) than Phosphor's 16.
- Open problem at pause: in CC the cheek line runs down inside the beard. Moving the beard start up the cheek ("sideburn") was rejected as a bad theory.

**How to apply:** resume from CC, not earlier variants; shape first, texture second. See [[feedback-design-iteration]].
