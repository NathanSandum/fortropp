---
name: feedback-never-print-credential-files
description: "never print any part of a credentials/config file, even \"redacted\"; only test field presence"
metadata:
  node_type: memory
  type: feedback
  originSessionId: e38e9579-8876-4e06-8048-11567f7460be
  modified: 2026-10-27T04:07:21.913Z
---

Never print contents of credential or tool-config files (`.env`, `~/.config/op/config`, cloud CLI configs, keychains), not even through a redaction regex. Check only that fields exist (`grep -c '^NAME='`, `jq 'has("x")'`), or let the user run the command themselves.

**Why:** 2026-10-26 a sed redaction of `~/.config/op/config` missed `accountKey` and printed the user's 1Password Secret Key into the transcript; they had to reset it.

**How to apply:** for anything holding secrets, never output values; when a secret-handling step needs the user's own session (e.g. manual `op signin`), hand them the exact commands instead of working around it. Related: [[feedback-use-edit-tools]].
