---
id: T-1401
name: "Rewrite termlink event broadcast as channel.post(broadcast:global) wrapper"
description: >
  Rewrite termlink event broadcast as channel.post(broadcast:global) wrapper

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-29T08:11:33Z
last_update: 2026-04-29T08:11:33Z
date_finished: null
---

# T-1401: Rewrite termlink event broadcast as channel.post(broadcast:global) wrapper

## Context

`crates/termlink-cli/src/commands/events.rs::cmd_broadcast` (line 173) is the
last large contributor to T-1166 entry-gate failure: 184 of 193 `event.broadcast`
audit-log entries are from this CLI without `TERMLINK_SESSION_ID` set. The hub
already mirrors every successful `event.broadcast` to channel topic
`broadcast:global` (T-1162), so subscribers on either path see the same envelope.

Migration: when the caller does NOT specify `--targets` (the dominant case —
zero usages in repo), call `channel.post(topic="broadcast:global", msg_type=topic, ...)`
directly. The wire shape mirrors hub-side T-1162 mirror exactly. On any error
(older hub, signing setup issue, etc.) fall back to `event.broadcast` so the
command remains functional across version skew.

When `--targets` IS specified, keep `event.broadcast` (the per-target fan-out
semantics have no clean channel-aware substitute, and zero in-repo callers use
this flag).

## Acceptance Criteria

### Agent
- [ ] `cmd_broadcast` in `events.rs:173` routes to `channel.post(broadcast:global)` when `targets.is_empty()`; falls back to legacy `event.broadcast` on any failure
- [ ] When `!targets.is_empty()`, behavior is unchanged — still calls `event.broadcast` (preserves per-target fan-out)
- [ ] Channel post envelope mirrors hub-side T-1162 mirror shape: `topic="broadcast:global"`, `msg_type=<original_topic>`, `payload_b64=<JSON-serialized payload>`, signed with local identity
- [ ] If `TERMLINK_SESSION_ID` is set, it goes into `metadata.from` so the hub's soft-lint can attribute the caller (replaces the previous `params.from` injection that only worked for event.broadcast)
- [ ] Human-mode display preserves the "Broadcast '<topic>': 1/1 succeeded" prefix (with new offset suffix) so existing operator habits aren't broken
- [ ] JSON-mode preserves the legacy keys (`topic`, `targeted`, `succeeded`, `failed`) PLUS adds new keys (`channel_topic`, `offset`) for callers that want richer telemetry
- [ ] On a hub with channel.post enabled (current fleet), running `termlink event broadcast smoke 'p:1'` produces ZERO new `event.broadcast` lines in `<runtime_dir>/rpc-audit.jsonl` and one new `channel.post` line
- [ ] cargo build / cargo test / cargo clippy clean for `termlink` and any other affected crates
- [ ] `termlink channel state broadcast:global` after a broadcast shows the new envelope with `msg_type=<original_topic>` (verifies the wire shape matches hub-side mirror behavior end-to-end)

## Verification

cargo build -p termlink 2>&1 | tail -3 | grep -qE "Finished"
cargo clippy -p termlink --tests -- -D warnings 2>&1 | tail -3 | grep -qE "Finished"
cargo test -p termlink 2>&1 | tail -25 | grep -qE "test result: ok\.\s+[0-9]+ passed"
grep -q "broadcast:global" crates/termlink-cli/src/commands/events.rs
grep -q "try_broadcast_via_channel_post\|channel.post" crates/termlink-cli/src/commands/events.rs

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

### 2026-04-29T08:11:33Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1401-rewrite-termlink-event-broadcast-as-chan.md
- **Context:** Initial task creation
