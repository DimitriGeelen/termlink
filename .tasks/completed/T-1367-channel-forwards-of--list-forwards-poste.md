---
id: T-1367
name: "channel forwards-of — list forwards posted by a sender"
description: >
  Add `channel forwards-of <topic> [sender]` — list every msg_type=forward
  envelope on <topic> whose sender_id matches [sender] (defaults to caller).
  Each row: forward_offset, forwarded_from (origin topic + offset),
  forwarded_sender (original poster), payload preview, ts. Reverse view
  parallel to `channel reactions-of` (T-1362) — answers "what has X
  cross-posted into this topic?".

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [agent-conversation, matrix, forwards, channel-cli]
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/main.rs]
related_tasks: [T-1362, T-1346]
created: 2026-04-28T09:50:00Z
last_update: 2026-04-28T10:11:42Z
date_finished: 2026-04-28T10:11:42Z
---

# T-1367: channel forwards-of — list forwards posted by a sender

## Context

Forwards (T-1346) carry `metadata.forwarded_from=<origin-topic>:<offset>` and
`metadata.forwarded_sender=<origin-sender>`. The forwarder's own identity sits
in the envelope's `sender_id`. There is no command to enumerate "forwards by
sender X into topic Y" — useful for audit ("who is amplifying what?") and for
reviewing one agent's curation behaviour.

Mirrors `channel reactions-of` shape: positional `<topic>`, optional `[sender]`
(defaults to caller fingerprint), `--hub`, `--json`. Honors redaction (redacted
forwards dropped). Sort: forward_offset desc (most recent first).

## Acceptance Criteria

### Agent
- [x] CLI variant `Channel ForwardsOf <topic> [sender]` accepted; `--hub`, `--json` flags wired
- [x] `compute_forwards_of` (pure helper) added with unit tests covering: no forwards (empty), single forward, multiple forwards sorted desc, forward by other sender excluded, redacted forward dropped, malformed `forwarded_from` (no colon) ignored
- [x] Live smoke test against the local hub — forward 2 posts as alice, run `forwards-of` filtered to alice → see both rows
- [x] e2e step 40 added (positive + negative + JSON shape + sender filter)
- [x] `cargo build --release -p termlink && cargo test -p termlink --bins --quiet && cargo clippy --all-targets --workspace -- -D warnings` all green

## Updates

### 2026-04-28T10:05Z — discriminator fix mid-build
- First helper draft filtered on `msg_type=forward`, but `cmd_channel_forward` preserves the **original** msg_type (e.g. "chat") and only sets the metadata pair. Smoke test exposed the bug; fix dropped the msg_type filter and relied on `extract_forward` (which already checks both metadata fields). Captured in helper doc-comment.

## Verification

cargo test -p termlink --bins --quiet 2>&1 | tail -3
cargo clippy --all-targets --workspace -- -D warnings 2>&1 | tail -3

## Decisions

## Updates

### 2026-04-28T09:50:00Z — task scoped
- ACs filled before any source-file edit (G-020 build-readiness gate).

### 2026-04-28T10:11:42Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
