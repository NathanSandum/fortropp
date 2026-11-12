---
name: feedback-cargo-add
description: "Add Rust dependencies with `cargo add`, never by hand-writing a version into Cargo.toml"
metadata:
  node_type: memory
  type: feedback
  originSessionId: fc2c38e4-5528-455b-8580-d61329f943c2
  modified: 2026-10-21T22:17:08.695Z
---

Add or re-add Rust dependencies with `cargo add` (with `--features`, `--no-default-features`, `--optional`, `--dev` as needed), never by typing a version into Cargo.toml.

**Why:** the model's remembered versions are stale; hand-written versions have picked outdated releases before, which frustrates the user.

**How to apply:** any new dependency or version change goes through `cargo add`; check the latest with `cargo search`/`cargo info` when discussing versions instead of quoting from memory. Same spirit as [[feedback-use-edit-tools]].
