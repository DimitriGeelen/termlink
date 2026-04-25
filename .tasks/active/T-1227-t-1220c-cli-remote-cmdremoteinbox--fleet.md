---
id: T-1227
name: "T-1220c: CLI remote cmd_remote_inbox_list migration (T-1220 wedge c, narrowed)"
description: >
  Migrate the List arm of cmd_remote_inbox_inner in crates/termlink-cli/src/commands/remote.rs (@1286) to use T-1231's list_with_fallback_with_client. Status / Clear arms and fleet-doctor inbox.status call deferred (same semantic blockers as T-1229 / T-1230).

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [T-1155, bus, migration, T-1220, wedge-c]
components: []
related_tasks: [T-1220, T-1225, T-1226, T-1228, T-1231]
created: 2026-04-25T07:00:14Z
last_update: 2026-04-25T08:40:00Z
date_finished: null
---

# T-1227: T-1220c CLI remote cmd_remote_inbox_list migration

## Context

Wedge-c consumer of T-1225 + T-1231. The List arm of
`cmd_remote_inbox_inner` was previously calling
`rpc_client.call("inbox.list", ...)` directly. With T-1231's
`list_with_fallback_with_client(&mut Client, host_port, ...)` entry point now
landed, the call site can swap in the capabilities-aware dispatcher.

Status / Clear arms and fleet-doctor's `inbox.status` call (@2810) stay on
legacy until T-1229 / T-1230 ship.

## Acceptance Criteria

### Agent
- [x] List arm calls `inbox_channel::list_with_fallback_with_client(&mut rpc_client, conn.hub, target, cache, ctx)`
- [x] Uses `hub_capabilities::shared_cache()` so the cache is shared with local CLI calls
- [x] Passes a fresh `FallbackCtx` (CLI process is short-lived)
- [x] `conn.hub` (profile name or host:port) used as the cache key
- [x] Display path preserved: `{id} — {filename} ({size} bytes)` per remote inbox formatting
- [x] JSON path preserves `{transfers: [...]}` wrapper for backward compat
- [x] No edits to Status arm, Clear arm, or fleet-doctor inbox.status (out of scope, see Decisions)
- [x] `cargo build -p termlink` clean
- [x] `cargo clippy -p termlink -- -D warnings` clean

## Verification

cargo build -p termlink 2>&1 | tail -5
cargo clippy -p termlink -- -D warnings 2>&1 | tail -5
grep -q "list_with_fallback_with_client" crates/termlink-cli/src/commands/remote.rs

## Decisions

### 2026-04-25 — Narrow scope from 4 sites to 1 site

- **Chose:** Migrate only the List arm under T-1227. Status / Clear arms +
  fleet-doctor remain on legacy `inbox.status` / `inbox.clear`.
- **Why:**
  - List arm: single target, returns transfer list — direct fit for
    `list_with_fallback_with_client` (added by T-1231).
  - Status arm + fleet-doctor: same aggregation issue as T-1229 (channel
    surface is per-topic; needs a new aggregation helper or stays on
    legacy until inception Q5 dual-read layer ships).
  - Clear arm: same Q4 spool-deletion semantic split as T-1230.
- **Rejected:** Bundling all 4 sites — same "one task = one deliverable"
  argument as T-1226 / T-1228.

## Updates

### 2026-04-25T07:00:14Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1227-t-1220c-cli-remote-cmdremoteinbox--fleet.md
- **Context:** Initial task creation

### 2026-04-25T08:33:44Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-04-25T08:40:00Z — scope-narrow [agent]
- **Change:** Renamed task to "cmd_remote_inbox_list migration"
- **Change:** Wrote real Agent ACs + verification commands
- **Reason:** Same scope-narrowing pattern as T-1226 / T-1228. T-1231 unblocked the helper-extension prerequisite.
