---
id: T-1234
name: "Hub: channel.trim(topic, before_offset?) router method (T-1230a)"
description: >
  Add hub-side channel.trim(topic, before_offset?) RPC method per T-1230 Option A. Destructive hub-side delete that mirrors legacy inbox.clear semantics (affects ALL subscribers). Replaces inbox.clear under T-1166 retirement. Pairs with channel.cursor.advance (separate sub-task) for the per-subscriber semantic.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [T-1230, T-1155, bus, channel, hub]
components: []
related_tasks: []
created: 2026-04-25T10:15:21Z
last_update: 2026-04-25T10:15:34Z
date_finished: null
---

# T-1234: Hub: channel.trim(topic, before_offset?) router method (T-1230a)

## Context

T-1230a per docs/reports/T-1230-inception.md. Foundation hub method for the inbox.clear migration. Adds destructive `channel.trim(topic, before_offset?)` that mirrors legacy `inbox.clear` semantics (deletes hub-side; affects ALL subscribers). Pairs with future `channel.cursor.advance` for the per-subscriber semantic. Index-only delete (log file bytes remain — same convention as existing `sweep_records`).

## Acceptance Criteria

### Agent
- [x] `Bus::trim_topic(topic, before_offset: Option<u64>) -> Result<u64>` added to `crates/termlink-bus/src/lib.rs` + thin `Meta::trim_records(topic, before_offset)` in `crates/termlink-bus/src/meta.rs`. `before_offset=None` deletes all records; `Some(N)` deletes records with `offset < N`. Returns count deleted. Unknown topic returns `Ok(0)`.
- [x] Protocol constant `CHANNEL_TRIM = "channel.trim"` added to `crates/termlink-protocol/src/control.rs`.
- [x] Hub router dispatches `"channel.trim"` to `crate::channel::handle_channel_trim` (mirrors existing channel.* dispatch pattern).
- [x] `handle_channel_trim` in `crates/termlink-hub/src/channel.rs`: requires `topic` param; optional `before_offset` (u64). Returns `{ok: true, deleted: N, topic: "..."}`. Missing topic returns `-32602`.
- [x] Hub method registry: add `"channel.trim"` to the methods list at router.rs:752+ so `hub.capabilities` advertises it.
- [x] Unit test in `termlink-bus`: post 5 records, trim with `before_offset=Some(3)` → 3 deleted, count=2; trim with `None` → 2 deleted, count=0; trim unknown topic → 0.
- [x] Unit test in `termlink-hub` channel.rs: post 3, full trim, assert response `{ok: true, deleted: 3}`; subsequent subscribe returns empty.
- [x] `cargo build -p termlink-bus -p termlink-hub -p termlink-protocol` clean (0 new warnings).
- [x] `cargo test -p termlink-bus -p termlink-hub -p termlink-protocol` passes.

## Verification

cargo build -p termlink-bus -p termlink-hub -p termlink-protocol 2>&1 | tail -5
cargo test -p termlink-bus trim 2>&1 | tail -10
cargo test -p termlink-hub --lib channel::tests::trim 2>&1 | tail -10
cargo test -p termlink-protocol channel_trim 2>&1 | tail -10

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

### 2026-04-25T10:15:21Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1234-hub-channeltrimtopic-beforeoffset-router.md
- **Context:** Initial task creation

### 2026-04-25T10:15:34Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
