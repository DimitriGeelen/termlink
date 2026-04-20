---
id: T-1162
name: "T-1155/5 Migrate event.broadcast callers → channel.post(topic=broadcast:global)"
description: >
  ~2 producer sites in events.rs + tools.rs. Wrap legacy method as channel.post adapter; preserve semantics. See T-1155 S-5 migration plan phase 2.

status: work-completed
workflow_type: refactor
owner: human
horizon: now
tags: [T-1155, bus, migration]
components: [crates/termlink-cli/tests/cli_integration.rs, crates/termlink-hub/src/channel.rs, crates/termlink-hub/src/router.rs]
related_tasks: [T-1155, T-1158]
created: 2026-04-20T14:12:10Z
last_update: 2026-04-20T22:31:41Z
date_finished: 2026-04-20T22:31:41Z
---

# T-1162: T-1155/5 Migrate event.broadcast callers → channel.post(topic=broadcast:global)

## Context

First migration in the T-1155 bus rollout (per T-1155 §"Migration strategy" Phase 2): expose every `event.broadcast` payload on the `channel.*` surface without forcing producer or receiver rewrites. Smallest migration surface (~2 producer sites per call-site audit), chosen first to prove the pattern.

Depends on: T-1160 (channel API shipped). Legacy `event.broadcast` stays working until T-1166 retires it.

**Call sites (audited 2026-04-21):**
- Producers: `crates/termlink-cli/src/commands/events.rs:201` (`termlink broadcast` verb), `crates/termlink-mcp/src/tools.rs:1744` (`termlink_broadcast` MCP tool)
- Hub handler: `crates/termlink-hub/src/router.rs::handle_event_broadcast`
- Tests exercising the path: `crates/termlink-hub/src/router.rs:1649/1699/1820/2095/2239`

**Scope note (2026-04-21):** The original ACs bundled three orthogonal wedges (dual-write shim, receiver rewrite, capabilities/telemetry). Split cleanly:
- **This task (T-1162) = server-side dual-write shim only.** Zero producer/receiver churn; pure additive — every `event.broadcast` payload is *also* appended to `channel:broadcast:global` so subscribers can migrate at their own pace.
- Receiver rewrites (`event.collect`/`event.poll` kind-filter → `channel.subscribe`) → T-1173 (new follow-up)
- Capabilities handshake for pre-upgrade peers → T-1131 (already captured, different wedge)
- Fleet-doctor broadcast migration telemetry → T-1132 (already captured)

## Acceptance Criteria

### Agent
- [x] Audit `event.broadcast` call sites (recorded in Context above)
- [x] Topic `broadcast:global` auto-created at hub startup with `Retention::Messages(1000)` via `channel::init_bus` (idempotent on name+policy)
- [x] Hub-side shim: `handle_event_broadcast` dual-writes each payload into `broadcast:global` via a new `channel::mirror_event_broadcast(topic, payload)` helper. Best-effort — logs on error, never blocks the existing fanout
- [x] Envelope shape for mirrored broadcasts: `sender_id = "hub:event.broadcast"`, `msg_type = <event-topic>`, `payload = serde_json::to_vec(&payload)`, no signature (hub-originated internal mirror — signature enforcement is on the `channel.post` RPC path only)
- [x] Router test `event_broadcast_mirrors_to_bus_topic`: call `handle_event_broadcast` with no targets → verify one envelope lands in `broadcast:global` with matching topic + payload
- [x] Router test `event_broadcast_mirror_is_non_blocking`: call shim with bus initialised, confirm mirror succeeds; no regression for the two existing broadcast router tests
- [x] `cargo build --workspace` + `cargo test -p termlink-hub --lib` (207+ tests) + `cargo clippy --workspace --lib --tests -- -D warnings` pass

### Human
- [ ] [REVIEW] Smoke-test a real dispatch cycle after the migration
  **Steps:**
  1. Run `termlink dispatch "echo hello"` against a local hub
  2. Confirm the worker runs, reports back, and `event.collect` still returns the exit code
  3. Run `termlink channel subscribe broadcast:global` in one terminal while dispatching in another — verify events are visible under both old (`event.collect`) and new (`channel.subscribe`) APIs
  **Expected:** Both paths observe the same events
  **If not:** Note which direction leaks; open a follow-up

## Verification

cargo build --workspace
cargo test -p termlink-hub --lib
cargo clippy --workspace --lib --tests -- -D warnings
grep -q "broadcast:global" crates/termlink-hub/src/channel.rs
grep -q "mirror_event_broadcast" crates/termlink-hub/src/router.rs
grep -q "mirror_event_broadcast" crates/termlink-hub/src/channel.rs

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

### 2026-04-20T14:12:10Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1162-t-11555-migrate-eventbroadcast-callers--.md
- **Context:** Initial task creation

### 2026-04-20T22:27:44Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)

### 2026-04-20T22:31:41Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
