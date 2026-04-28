---
id: T-1366
name: "channel edits-of — render edit history for a target offset (m.replace)"
description: >
  Add `channel edits-of <topic> <offset>` — show the original post at <offset>
  followed by every `msg_type=edit` envelope (`metadata.replaces=<offset>`)
  in chronological order (oldest edit first). Each row: edit_offset, sender,
  ts_ms, payload preview. Matrix m.replace history analog. Skips redacted edits.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [agent-conversation, matrix, edits, channel-cli]
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/main.rs]
related_tasks: [T-1321, T-1322]
created: 2026-04-28T09:30:00Z
last_update: 2026-04-28T09:48:51Z
date_finished: 2026-04-28T09:48:51Z
---

# T-1366: channel edits-of — render edit history for a target offset (m.replace)

## Context

`channel edit <topic> <offset> <text>` (T-1321) emits a replace envelope but
clients can only ever see the latest text — there is no command to enumerate
the full edit chain. `channel edits-of` fills that gap. Useful for audit
("what was the original wording?"), thread review, and forensic correlation.

Pure helper `compute_edits_of(envelopes, target) -> Option<EditsOfReport>`
where the report is `{ original: EditRow, edits: Vec<EditRow> }` (None when
the target itself is missing or redacted). Honors redaction set: redacted
edits dropped. Sort: edits ascending by ts_ms then by edit_offset.

## Acceptance Criteria

### Agent
- [x] CLI variant `Channel EditsOf <topic> <offset>` accepted; `--hub`, `--json` flags wired
- [x] `compute_edits_of` (pure helper) added with unit tests covering: target with no edits (single-row report), target with multiple edits (chronological order), redacted edit dropped, redacted target → None, edit with non-numeric `replaces` ignored, edits to other targets ignored
- [x] Live smoke test against the local hub produces correct output (post + 2 edits → 3 rows)
- [x] e2e step 39 added to `tests/e2e/agent-conversation.sh` (positive + negative + JSON shape)
- [x] `cargo build --release -p termlink && cargo test -p termlink --bins --quiet && cargo clippy --all-targets --workspace -- -D warnings` all green

## Verification

cargo test -p termlink --bins --quiet 2>&1 | tail -3
cargo clippy --all-targets --workspace -- -D warnings 2>&1 | tail -3

## Decisions

## Updates

### 2026-04-28T09:30:00Z — task scoped
- ACs filled before any source-file edit (G-020 build-readiness gate).

### 2026-04-28T09:48:51Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
