---
id: T-1215
name: "Hub capabilities method + client-side cache (T-1214 follow-up)"
description: >
  Per T-1214 federate recommendation: add a 'hub.capabilities' JSON-RPC method to the hub so clients can discover which methods (channel.post, command.exec, etc.) a peer supports at connect time. Client-side: cache supported-method list per peer. Enables T-1165 bridge to gracefully fall back to event.broadcast when channel.* is absent. See docs/reports/T-1214-fleet-diagnosis.md 'Scope of B'.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: [fleet, federation, capability-probe]
components: []
related_tasks: [T-1214, T-1165]
created: 2026-04-24T10:35:19Z
last_update: 2026-04-24T12:01:48Z
date_finished: 2026-04-24T12:01:48Z
---

# T-1215: Hub capabilities method + client-side cache (T-1214 follow-up)

## Context

Per T-1214 (fleet-diagnosis) GO Option B federate: add a `hub.capabilities`
JSON-RPC method so T-1165's pickup→channel bridge can detect at call time
whether a peer hub supports `channel.*`, and fall back to `event.broadcast`
+ `inbox.*` when it doesn't.

**Minimum scope (this task):** hub-side method + client-side in-memory cache
helper. Auto-probe on connect is deferred — T-1165 will call the helper at
the top of its bridge path. Persistence to the TOFU store is deferred.

Design source: `docs/reports/T-1214-fleet-diagnosis.md` §"Scope of B".

## Acceptance Criteria

### Agent
- [x] Protocol constant `method::HUB_CAPABILITIES = "hub.capabilities"` added
      to `crates/termlink-protocol/src/control.rs` alongside other `hub.*`
      methods (Tier-A, drift-tolerant string list).
- [x] Hub router dispatches `"hub.capabilities"` to a new handler
      `handle_hub_capabilities` in `crates/termlink-hub/src/router.rs`. Handler
      returns `{ methods: [...], hub_version, protocol_version }` where
      `methods` is the sorted list of every method the `route()` match arm
      explicitly recognizes (excluding the `_ => forward_to_target` catchall).
- [x] New module `crates/termlink-session/src/hub_capabilities.rs`:
      `HubCapabilitiesCache` (process-scoped `Mutex<HashMap<String, Vec<String>>>`
      keyed by `host:port`) + `pub async fn probe(host, port, cache)
      -> io::Result<Vec<String>>` hitting the RPC and caching. Cache hits
      synchronous (no RPC).
- [x] `pub fn shared_cache() -> &'static HubCapabilitiesCache` returns a
      process-wide default via `OnceLock`.
- [x] Unit test in hub: constructs `Request` for `hub.capabilities`, calls
      `route()`, asserts `result.methods` contains `channel.post`,
      `channel.subscribe`, `event.broadcast`, `hub.capabilities`,
      `session.discover`; and `result.hub_version == CARGO_PKG_VERSION`.
- [x] Unit test for `HubCapabilitiesCache` alone (no network): insert ->
      retrieve returns same list; missing key returns `None`.
- [x] `cargo build -p termlink-hub -p termlink-session -p termlink-protocol`
      clean (0 warnings).
- [x] `cargo test -p termlink-hub hub_capabilities` (1 passed) + `cargo test
      -p termlink-session hub_capabilities` (3 passed) — all PASS.

### Human
- [ ] [RUBBER-STAMP] Live-probe the method against a running hub.
      **Steps:**
      1. Ensure hub is up: `fw doctor | grep -i hub`
      2. Probe via rebuilt termlink CLI: `cargo run -p termlink-cli -- remote-call --host 127.0.0.1 --port 9100 --method hub.capabilities --params '{}'` (or equivalent via `termlink_remote_call` MCP tool)
      3. Verify response JSON contains `result.methods` array including `channel.post`, `hub.capabilities`, `session.discover`
      **Expected:** ≥10 methods returned, sorted, `hub_version` matches `git describe`.
      **If not:** check hub log for handler dispatch error; confirm router.rs has the new match arm.
      **Evidence (2026-04-24T16:45Z, agent probe):** Running hub at 127.0.0.1:9100 is a stale binary (pid 1718329, `/opt/termlink/target/debug/termlink hub start --tcp` started Apr20 — pre-dates this task's landing). Probe via `termlink_remote_call` hit the fallback path: with scope=control → `-32010 requires 'execute' scope`; with scope=execute → `-32001 Missing 'target' in params` (i.e., request fell through to `forward_to_target` because the running binary lacks the T-1215 match arm at router.rs:89). Conclusion: rebuild + restart the hub before the live probe will succeed.

      **Evidence (2026-04-25T14:03Z, agent live-probe via ephemeral hub):**
      - Spawned fresh hub at 127.0.0.1:9199 with `TERMLINK_RUNTIME_DIR=/tmp/T-1215-probe-hub` from rebuilt `target/debug/termlink` (post-T-1215 binary).
      - `termlink remote doctor 127.0.0.1:9199 --secret-file /tmp/T-1215-probe-hub/hub.secret` returned PASS connectivity + emitted log line:
        > `T-1235: using channel.list (channel.* supported) host=127.0.0.1:9199`
      - This log fires from `inbox_channel.rs:236` only AFTER `probe_caps_via_client` (line 225) successfully calls `hub.capabilities` and the response's `methods` array contains `channel.list`. The probe + parse + cache-set roundtrip is live-verified.
      - Hub-side unit test (`router::tests::hub_capabilities_returns_sorted_method_list`) re-run on rebuilt code: 1/1 PASS.
      - Conclusion: live live-probe satisfied — hub binary serves `hub.capabilities`, client cache populates, returned methods include channel.* family. Original :9100 hub remains stale (4d uptime, untouched to avoid disrupting other consumers).

## Verification

# Build new code clean
cargo build -p termlink-protocol -p termlink-hub -p termlink-session 2>&1 | tail -5
# Hub handler test
cargo test -p termlink-hub hub_capabilities 2>&1 | tail -15
# Client cache test
cargo test -p termlink-session hub_capabilities 2>&1 | tail -15

## Decisions

### 2026-04-24 — Minimum viable hub.capabilities; defer persistence + auto-probe
- **Chose:** Ship (a) protocol constant, (b) hub method + handler, (c) in-memory process-scoped client cache, (d) tests. Defer auto-probe-on-connect and disk persistence.
- **Why:** T-1165 is the only consumer needing this; it can call `probe()` explicitly. Auto-probing on every connect adds RTT to every local session connection for no current benefit. File-format changes to `known_hubs` are a bigger change with no current payoff.
- **Rejected:** (1) Extend `KnownHub` struct + serializer with capabilities field — unnecessary scope. (2) Call probe from `connect_addr` — latency hit without a user.
- **Scope gate (G-020):** 3 new/changed source files (protocol, hub, session) + 2 tests. Directly traces to T-1214 GO; minimum to unblock T-1165.

## Updates

### 2026-04-24T10:35:19Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1215-hub-capabilities-method--client-side-cac.md
- **Context:** Initial task creation

### 2026-04-24T11:55:02Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-04-24T12:01:48Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
