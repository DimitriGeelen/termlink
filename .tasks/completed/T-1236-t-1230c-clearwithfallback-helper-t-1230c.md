---
id: T-1236
name: "T-1230c clear_with_fallback helper (T-1230c critical dep for migration sites)"
description: >
  T-1230c clear_with_fallback helper (T-1230c critical dep for migration sites)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/infrastructure.rs, crates/termlink-mcp/src/tools.rs, crates/termlink-session/src/inbox_channel.rs]
related_tasks: []
created: 2026-04-25T10:31:08Z
last_update: 2026-04-25T10:46:52Z
date_finished: 2026-04-25T10:46:52Z
---

# T-1236: T-1230c clear_with_fallback helper (T-1230c critical dep for migration sites)

## Context

T-1230c per inception report (`docs/reports/T-1230-inception.md`): build the
`clear_with_fallback{,_with_client}` helper in `crates/termlink-session/src/inbox_channel.rs`.
Mirrors T-1235's `status_with_fallback` pattern (same FallbackCtx, same
HubCapabilitiesCache probe, same warn-once semantics). This is the critical
dependency for the four `inbox.clear` call-site migrations (T-1230d-g).

Returns `InboxClearResult { cleared, target }` matching legacy
`inbox.clear` reply shape so call sites can swap `inbox.clear` →
`clear_with_fallback` without touching display code.

## Acceptance Criteria

### Agent
- [x] `pub struct InboxClearResult { cleared: u64, target: String }` exported from `inbox_channel.rs`
- [x] `clear_with_fallback(addr, target_or_all, cache, ctx) -> io::Result<InboxClearResult>` builds on top of `clear_with_fallback_with_client`
- [x] `clear_with_fallback_with_client(client, host_port, target_or_all, cache, ctx) -> io::Result<InboxClearResult>` probes capabilities, dispatches to `channel.trim` when `CHANNEL_TRIM` is advertised, falls back to `inbox.clear` on `-32601`
- [x] Single-target path: trims topic `inbox:<target>` → returns `{cleared: deleted, target}`
- [x] All-targets path: enumerates `channel.list(prefix="inbox:")`, trims each, returns `{cleared: sum, target: "all"}`
- [x] Warn-once on first channel/legacy use per (host_port, kind), via existing `FallbackCtx::warn_once`
- [x] Method-not-found on `channel.trim` flags peer legacy-only via `FallbackCtx::flag_legacy_only`
- [x] Tests: aggregate-from-channel-list edge cases (empty, missing keys, mixed prefix) — pure-fn style like `aggregate_status_from_channel_list`
- [x] `cargo build -p termlink-session` clean
- [x] `cargo test -p termlink-session --lib inbox_channel::` passes

## Verification
cargo build -p termlink-session 2>&1 | tail -3 | grep -q -E "Finished|warning: unused"
cargo test -p termlink-session --lib inbox_channel:: 2>&1 | tail -5 | grep -q "test result: ok"

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

### 2026-04-25T10:31:08Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1236-t-1230c-clearwithfallback-helper-t-1230c.md
- **Context:** Initial task creation

### 2026-04-25T10:46:52Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
