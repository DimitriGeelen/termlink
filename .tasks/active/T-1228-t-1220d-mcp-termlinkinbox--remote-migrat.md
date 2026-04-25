---
id: T-1228
name: "T-1220d: MCP termlink_inbox_list migration (T-1220 wedge d, narrowed)"
description: >
  Migrate termlink_inbox_list MCP tool in crates/termlink-mcp/src/tools.rs (@4564) to use T-1225's list_with_fallback helper. termlink_inbox_status / termlink_inbox_clear (Q4 split) and termlink_remote_inbox_* (need auth-client helper extension, see T-1231) deferred — see Decisions.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [T-1155, bus, migration, T-1220, wedge-d]
components: []
related_tasks: [T-1220, T-1225, T-1226, T-1231]
created: 2026-04-25T07:00:17Z
last_update: 2026-04-25T08:30:00Z
date_finished: null
---

# T-1228: T-1220d MCP termlink_inbox_list migration

## Context

Wedge-d consumer of T-1225, MCP variant. Mirrors T-1226's CLI-local pattern
exactly — `termlink_inbox_list` calls `rpc_call(&hub_socket, "inbox.list",
{target})` against the local hub, returns transfer list as JSON string.

Same scope-narrowing as T-1226 + an extra deferral for the remote variants
which need a helper extension (T-1231).

## Acceptance Criteria

### Agent
- [x] termlink_inbox_list calls `inbox_channel::list_with_fallback` instead of `rpc_call("inbox.list", ...)`
- [x] Uses `TransportAddr::unix(&hub_socket)` for the local hub socket
- [x] Uses `hub_capabilities::shared_cache()` so cache survives across MCP tool calls
- [x] Passes a fresh `FallbackCtx` per invocation (MCP tool calls are stateless)
- [x] JSON output preserves the `{transfers: [...]}` envelope shape consumers expect
- [x] No edits to termlink_inbox_status, termlink_inbox_clear, or termlink_remote_inbox_* (out of scope, see Decisions)
- [x] `cargo build -p termlink-mcp` clean
- [x] `cargo clippy -p termlink-mcp -- -D warnings` clean

## Verification

cargo build -p termlink-mcp 2>&1 | tail -5
cargo clippy -p termlink-mcp -- -D warnings 2>&1 | tail -5
grep -q "list_with_fallback" crates/termlink-mcp/src/tools.rs

## Decisions

### 2026-04-25 — Narrow scope from 6 sites to 1 site

- **Chose:** Migrate only `termlink_inbox_list` under T-1228. Defer status /
  clear (T-1229 / T-1230) and `termlink_remote_inbox_*` (3 sites, blocked on
  T-1231) as follow-ups.
- **Why:**
  - termlink_inbox_list: single target, returns transfer list — direct fit
    for `list_with_fallback`. Same shape as cmd_inbox_list (T-1226).
  - termlink_inbox_status / clear: same semantic blockers as T-1229 / T-1230.
  - termlink_remote_inbox_*: helper currently takes `&TransportAddr` and
    builds unauthenticated connections; remote MCP tools authenticate via
    `connect_remote_hub_mcp` which yields an `RpcClient`. Helper needs
    extension before these can migrate (T-1231).
- **Rejected:** Bundling all 6 sites — same "one task = one deliverable"
  argument as T-1226, plus the auth blocker for the remote half.

## Updates

### 2026-04-25T07:00:17Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1228-t-1220d-mcp-termlinkinbox--remote-migrat.md
- **Context:** Initial task creation

### 2026-04-25T08:30:00Z — scope-narrow [agent]
- **Change:** Renamed task to "termlink_inbox_list migration" (was 6-site bundle)
- **Change:** Wrote real Agent ACs + verification commands
- **Reason:** Same semantic split as T-1226 (status aggregation, clear deletion). Remote variants additionally blocked on helper extension (T-1231).
