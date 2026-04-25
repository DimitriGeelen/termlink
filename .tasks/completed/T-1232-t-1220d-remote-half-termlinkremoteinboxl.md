---
id: T-1232
name: "T-1220d remote half: termlink_remote_inbox_list MCP migration"
description: >
  Migrate termlink_remote_inbox_list MCP tool in crates/termlink-mcp/src/tools.rs (@4720) to use T-1231's list_with_fallback_with_client. Same pattern as T-1227 but for the MCP wrapper.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [T-1155, bus, migration, T-1220, wedge-d-remote]
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: [T-1220, T-1225, T-1227, T-1228, T-1231]
created: 2026-04-25T08:35:43Z
last_update: 2026-04-25T08:37:42Z
date_finished: 2026-04-25T08:37:42Z
---

# T-1232: T-1220d remote half — termlink_remote_inbox_list MCP migration

## Context

Wedge-d remote-half consumer of T-1231. The `termlink_remote_inbox_list`
MCP tool was previously calling `rpc_client.call("inbox.list", ...)`
directly. With T-1231's `list_with_fallback_with_client(&mut Client,
host_port, ...)` entry point now landed, the MCP tool can swap in the
capabilities-aware dispatcher.

Status / Clear remote MCP tools stay on legacy until T-1229 / T-1230 ship.

## Acceptance Criteria

### Agent
- [x] termlink_remote_inbox_list calls `inbox_channel::list_with_fallback_with_client(&mut rpc_client, &p.hub, &p.target, cache, ctx)`
- [x] Uses `hub_capabilities::shared_cache()` so the cache is shared with CLI calls
- [x] Passes a fresh `FallbackCtx` (MCP tool calls are stateless)
- [x] `p.hub` (profile name or host:port) used as the cache key
- [x] JSON output preserves `{ok: true, hub, result: {transfers: [...]}}` envelope shape
- [x] No edits to termlink_remote_inbox_status or termlink_remote_inbox_clear (out of scope)
- [x] `cargo build -p termlink-mcp` clean
- [x] `cargo clippy -p termlink-mcp -- -D warnings` clean

## Verification

cargo build -p termlink-mcp 2>&1 | tail -5
cargo clippy -p termlink-mcp -- -D warnings 2>&1 | tail -5
grep -q "list_with_fallback_with_client" crates/termlink-mcp/src/tools.rs

## Decisions

### 2026-04-25 — Mirrors T-1227 pattern

- **Chose:** Same scope-narrowing as T-1227 (List arm only). MCP-remote
  Status / Clear stay on legacy until the broader status/clear redesign
  ships (T-1229 / T-1230).
- **Why:** Identical reasoning — list maps cleanly, status needs aggregation,
  clear has Q4 semantic split.

## Updates

### 2026-04-25T08:35:43Z — task-created [agent]
- **Action:** Created task to capture MCP-remote half of wedge-d
- **Reason:** T-1228 (work-completed) was scoped to local MCP only. Remote MCP needs its own task ID for traceability and ACs.

### 2026-04-25T08:37:42Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
