---
id: T-1231
name: "T-1220 follow-up: extend list_with_fallback to accept authenticated Client"
description: >
  Refactor T-1225 inbox_channel helper so the dispatch body operates on `&mut Client` (works equally for unauth Unix/TCP and post-auth Client). Add public entry `list_with_fallback_with_client(client, host_port, target, cache, ctx)` for callers who already authenticated. Original `list_with_fallback(addr, ...)` becomes a thin wrapper. Unblocks T-1227 (CLI remote) and remote half of T-1228 (MCP).

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [T-1155, bus, migration, T-1220, T-1225-followup]
components: [crates/termlink-mcp/src/tools.rs, crates/termlink-session/src/inbox_channel.rs]
related_tasks: [T-1220, T-1225, T-1227, T-1228]
created: 2026-04-25T08:27:29Z
last_update: 2026-04-25T08:33:32Z
date_finished: 2026-04-25T08:33:32Z
---

# T-1231: extend list_with_fallback to accept authenticated Client

## Context

T-1225 helper takes `&TransportAddr` and opens fresh unauth connections via
`Client::connect_addr` for each of probe / subscribe / list. Wedge-c
(`cmd_remote_inbox_*`) and the remote half of wedge-d
(`termlink_remote_inbox_*`) authenticate via `connect_remote_hub` /
`connect_remote_hub_mcp` which return an *already-authenticated* `Client`.
Reopening + re-authenticating per call is wasteful and brittle.

Refactor: factor the dispatch body into helpers that take `&mut Client +
host_port`. Original entry stays for the unauth case (Unix socket, untrusted
TCP probe) but now opens *one* Client for the whole probe→dispatch sequence.

## Acceptance Criteria

### Agent
- [x] New public fn `list_with_fallback_with_client(client: &mut Client, host_port: &str, target: &str, cache: &HubCapabilitiesCache, ctx: &mut FallbackCtx) -> io::Result<Vec<InboxEntry>>`
- [x] Internal `probe_caps_via_client`, `call_channel_subscribe_via_client`, `call_legacy_inbox_list_via_client` private helpers added
- [x] Probe path checks `cache.get(host_port)` first; on miss, calls `client.call("hub.capabilities", ...)` then `cache.set(host_port, methods)`
- [x] Channel-subscribe path detects method-not-found via `RpcResponse::Error.code == -32601` (same logic as addr variant)
- [x] Existing `list_with_fallback(addr, ...)` becomes a thin wrapper: opens `Client::connect_addr(addr)`, derives `host_port_str(addr)`, delegates
- [x] Existing 7 unit tests still pass (no behavioral regression for the addr entry)
- [x] No new external dependencies added
- [x] `cargo build -p termlink-session` clean
- [x] `cargo test -p termlink-session inbox_channel` — 7 pass
- [x] `cargo clippy -p termlink-session -- -D warnings` clean

## Verification

cargo build -p termlink-session 2>&1 | tail -5
cargo test -p termlink-session inbox_channel 2>&1 | tail -10
cargo clippy -p termlink-session -- -D warnings 2>&1 | tail -5
grep -q "list_with_fallback_with_client" crates/termlink-session/src/inbox_channel.rs

## Decisions

### 2026-04-25 — Refactor strategy

- **Chose:** Factor dispatch into private `*_via_client` helpers that take
  `&mut Client`. Expose two public entries (addr-based wrapper + client-based
  direct).
- **Why:**
  - Cleanest separation: connection lifecycle owned by caller for the auth
    case (single connection reused for probe + dispatch), framework owns it
    for the unauth case (existing behavior preserved).
  - Avoids the trait/closure complexity of dependency-injecting an RPC
    dispatcher in async Rust (lifetime + send-bound friction).
  - Single connection per logical call is also a perf win for the addr path
    (3 connects → 1 connect today).
- **Rejected:**
  - **Trait `InboxRpc`**: extra abstraction, async-trait friction, no second
    consumer in sight to justify the indirection.
  - **Closure-based dispatcher**: Rust async closure ergonomics still rough;
    callers would have to write boilerplate.
  - **Two parallel public APIs without shared body**: ~80 LOC duplication;
    bug-divergence risk.

## Updates

### 2026-04-25T08:27:29Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1231-t-1220-follow-up-extend-listwithfallback.md
- **Context:** Initial task creation

### 2026-04-25T08:35:00Z — scope-and-acs [agent]
- **Change:** Wrote real Agent ACs + verification commands
- **Change:** Recorded refactor strategy in Decisions

### 2026-04-25T08:33:32Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
