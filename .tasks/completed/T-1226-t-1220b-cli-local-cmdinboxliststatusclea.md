---
id: T-1226
name: "T-1220b: CLI local cmd_inbox_list migration (T-1220 wedge b, narrowed)"
description: >
  Migrate cmd_inbox_list in crates/termlink-cli/src/commands/infrastructure.rs (@839) to use T-1225's list_with_fallback helper. cmd_inbox_status (aggregation) and cmd_inbox_clear (Q4 spool-deletion semantics) split out as follow-ups — see Decisions section.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [T-1155, bus, migration, T-1220, wedge-b]
components: [crates/termlink-cli/src/commands/infrastructure.rs, crates/termlink-session/src/inbox_channel.rs, crates/termlink-session/src/lib.rs]
related_tasks: [T-1220, T-1225]
created: 2026-04-25T07:00:11Z
last_update: 2026-04-25T08:26:14Z
date_finished: 2026-04-25T08:26:14Z
---

# T-1226: T-1220b CLI local cmd_inbox_list migration

## Context

Wedge-b consumer of T-1225. `cmd_inbox_list` is the cleanest of the three call
sites — single target, returns transfer list, maps directly to
`list_with_fallback(addr, target, cache, ctx)`.

`cmd_inbox_status` (aggregates across all targets) and `cmd_inbox_clear`
(Q4 spool-deletion semantics) need their own scope discussion before
migration. Captured as separate follow-ups (see Decisions).

## Acceptance Criteria

### Agent
- [x] cmd_inbox_list calls `inbox_channel::list_with_fallback` instead of `rpc_call("inbox.list", ...)`
- [x] Uses `TransportAddr::unix(&hub_socket)` for the local hub socket
- [x] Uses `hub_capabilities::shared_cache()` so cache is shared with future call sites
- [x] Passes a fresh `FallbackCtx` (CLI process is short-lived; no need for persistence)
- [x] Display path (`{id} {file} ({size} bytes, {status})`) preserved — uses InboxEntry fields directly
- [x] JSON output path preserved — serializes `Vec<InboxEntry>` under a `transfers` key for backward compat
- [x] No edits to cmd_inbox_status or cmd_inbox_clear (out of scope, see Decisions)
- [x] `cargo build -p termlink` clean
- [x] `cargo clippy -p termlink -- -D warnings` clean

## Verification

cargo build -p termlink 2>&1 | tail -5
cargo clippy -p termlink -- -D warnings 2>&1 | tail -5
grep -q "list_with_fallback" crates/termlink-cli/src/commands/infrastructure.rs

## Decisions

### 2026-04-25 — Narrow scope from 3 sites to 1 site

- **Chose:** Migrate only `cmd_inbox_list` under T-1226. Capture
  `cmd_inbox_status` (T-1229) and `cmd_inbox_clear` (T-1230) as separate
  follow-ups.
- **Why:**
  - `cmd_inbox_list`: single target, returns transfer list — direct fit for
    `list_with_fallback`.
  - `cmd_inbox_status`: aggregates across ALL targets via
    `{total_transfers, targets[]}`. Channel surface is per-topic; needs a new
    aggregation helper (or stays on legacy until inception Q5 dual-read
    layer ships).
  - `cmd_inbox_clear`: legacy clear deletes spool files on hub disk. Per
    inception Q4, channel-backed clear advances local cursor only and does
    NOT mutate hub state. Semantic change requires explicit design discussion
    before edits land.
- **Rejected:** Bundling all 3 in one commit — violates "one task = one
  deliverable" and forces an all-or-nothing semantic decision on clear.

## Updates

### 2026-04-25T07:00:11Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1226-t-1220b-cli-local-cmdinboxliststatusclea.md
- **Context:** Initial task creation

### 2026-04-25T08:22:20Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-04-25T08:25:00Z — scope-narrow [agent]
- **Change:** Renamed task to "cmd_inbox_list migration" (was "{list,status,clear}")
- **Change:** Wrote real Agent ACs + verification commands
- **Reason:** cmd_inbox_status (aggregation) and cmd_inbox_clear (Q4 semantic split) need separate scope discussion. T-1229/T-1230 captured as follow-ups.

### 2026-04-25T08:26:14Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
