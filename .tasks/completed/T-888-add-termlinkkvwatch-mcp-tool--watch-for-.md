---
id: T-888
name: "Add termlink_kv_watch MCP tool — watch for key-value changes on a session"
description: >
  Add termlink_kv_watch MCP tool — watch for key-value changes on a session

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-05T07:27:29Z
last_update: 2026-04-23T15:32:22Z
date_finished: 2026-04-23T15:32:22Z
---

# T-888: Add termlink_kv_watch MCP tool — watch for key-value changes on a session

## Context

Existing KV store (`kv.set`, `kv.get`, `kv.list`, `kv.delete` in `handler.rs`)
has no change notification. Consumers must poll. The session already has an
EventBus with `event.subscribe` long-poll support. Approach: emit `kv.change`
events from `handle_kv_set`/`handle_kv_delete`, then expose a thin MCP wrapper
`termlink_kv_watch` that calls `event.subscribe` with `topic="kv.change"`. No
new RPC method — reuses existing subscribe infrastructure.

## Acceptance Criteria

### Agent
- [x] `handle_kv_set` emits `kv.change` event with payload `{key, value, op:"set", replaced}` on the session EventBus
- [x] `handle_kv_delete` emits `kv.change` event with payload `{key, value:null, op:"delete", deleted}` on the session EventBus
- [x] Events only emitted on actual state change (kv.delete on missing key still emits with deleted=false — keep simple, symmetrical with set)
- [x] New MCP tool `termlink_kv_watch` in `crates/termlink-mcp/src/tools.rs` calls `event.subscribe` with `topic="kv.change"`; accepts `target`, optional `timeout_ms`, optional `since`
- [x] `termlink_kv_watch` appears in help-text category `kv`
- [x] Integration test in `crates/termlink-session/tests/integration.rs`: kv.set emits kv.change event observable via event.subscribe
- [x] Integration test: kv.delete emits kv.change event with op=delete
- [x] `cargo build --workspace` and `cargo test --workspace` succeed

### Human
<!-- Agent-only task; no human verification required -->

## Verification

cargo build --workspace --quiet
cargo test --workspace --quiet --lib
grep -q "kv\.change" crates/termlink-session/src/handler.rs
grep -q "termlink_kv_watch" crates/termlink-mcp/src/tools.rs

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

### 2026-04-05T07:27:29Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-888-add-termlinkkvwatch-mcp-tool--watch-for-.md
- **Context:** Initial task creation

### 2026-04-05T07:27:45Z — status-update [task-update-agent]
- **Change:** status: started-work → issues
- **Reason:** Requires server-side KV watch mechanism — too complex for this session

### 2026-04-05T07:27:53Z — status-update [task-update-agent]
- **Change:** horizon: now → later

### 2026-04-22T04:52:51Z — status-update [task-update-agent]
- **Change:** horizon: later → next

### 2026-04-23T15:26:33Z — status-update [task-update-agent]
- **Change:** status: issues → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-04-23T15:32:22Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
