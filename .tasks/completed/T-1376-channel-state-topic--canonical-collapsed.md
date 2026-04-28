---
id: T-1376
name: "channel state <topic> — canonical collapsed view (edits applied, redactions hidden) — Matrix m.replace + m.redaction render"
description: >
  channel state <topic> — canonical collapsed view (edits applied, redactions hidden) — Matrix m.replace + m.redaction render

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-28T14:05:23Z
last_update: 2026-04-28T14:20:36Z
date_finished: 2026-04-28T14:20:36Z
---

# T-1376: channel state <topic> — canonical collapsed view (edits applied, redactions hidden) — Matrix m.replace + m.redaction render

## Context

`channel state <topic>` renders the canonical post-state of a topic — Matrix-style render
where `m.replace` events have been applied (latest edit text wins per parent) and
`m.redaction`-targeted offsets are hidden. This is THE Matrix concept that consumers
expect: a room-view that reflects current truth, not raw history.

Distinct from `channel subscribe` (raw envelope stream), `channel info` (synthesized
summary, T-1324), `channel edits-of` (single-target edit history, T-1366), and
`channel edit-stats` (topic-wide rollup, T-1375). This is the WALK-AND-COLLAPSE
view — one row per visible content message, post-edit, redactions removed.

## Acceptance Criteria

### Agent
- [x] `compute_state(envelopes, include_redacted)` pure helper exists in `commands/channel.rs`
- [x] `StateRow` struct holds {offset, sender_id, payload (post-edit), is_edited, edit_count, latest_edit_ts_ms, ts_ms, is_redacted}
- [x] Filter rule: skip meta envelopes (`UNREAD_META_TYPES`) — only content rows surface
- [x] Edit collapse: when target has edits, payload = latest edit's text (max ts, offset asc tiebreak); is_edited=true; edit_count>=1
- [x] Redaction rule: when not include_redacted, redacted offsets are dropped entirely; with flag, payload="[REDACTED]" and is_redacted=true
- [x] Sort order: offset asc (chronological)
- [x] `cmd_channel_state` async wrapper renders human + JSON output
- [x] At least 6 unit tests covering: empty topic, single message, message with one edit (collapsed), message with two edits (latest wins), redacted hidden, redacted shown with --include-redacted
- [x] `ChannelAction::State` variant in `cli.rs` with `topic`, optional `--include-redacted`, `--hub`, `--json`
- [x] Dispatch arm in `main.rs`
- [x] E2E step in `tests/e2e/agent-conversation.sh` exercises positive + redaction cases + JSON shape
- [x] Doc section added to `docs/operations/agent-conversations.md`
- [x] `cargo build --release -p termlink` passes
- [x] `cargo test -p termlink --bins` passes
- [x] `cargo clippy --all-targets --workspace -- -D warnings` clean

## Verification
cargo test -p termlink --bins --quiet 2>&1 | tail -5
cargo clippy --all-targets --workspace -- -D warnings 2>&1 | tail -5

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Updates

### 2026-04-28T14:05:23Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1376-channel-state-topic--canonical-collapsed.md
- **Context:** Initial task creation

### 2026-04-28T14:20:36Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
