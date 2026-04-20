---
id: T-1162
name: "T-1155/5 Migrate event.broadcast callers → channel.post(topic=broadcast:global)"
description: >
  ~2 producer sites in events.rs + tools.rs. Wrap legacy method as channel.post adapter; preserve semantics. See T-1155 S-5 migration plan phase 2.

status: captured
workflow_type: refactor
owner: agent
horizon: later
tags: [T-1155, bus, migration]
components: []
related_tasks: [T-1155, T-1158]
created: 2026-04-20T14:12:10Z
last_update: 2026-04-20T14:12:10Z
date_finished: null
---

# T-1162: T-1155/5 Migrate event.broadcast callers → channel.post(topic=broadcast:global)

## Context

First migration in the T-1155 bus rollout (per T-1155 §"Migration strategy" Phase 2): `event.broadcast` callers move to `channel.post(topic="broadcast:global")`. Smallest migration surface (~5 call sites per T-1155 §"Subsumption mapping"), chosen first to prove the pattern.

Depends on: T-1160 (channel API shipped). Legacy `event.broadcast` stays working until T-1166 retires it.

## Acceptance Criteria

### Agent
- [ ] Audit all current callers of `event.broadcast` — `grep -rn "event.broadcast\|event_broadcast" crates/ lib/` produces the exhaustive list; add it to this task file under "Call sites"
- [ ] Each caller rewritten to `channel.post(topic="broadcast:global", msg_type=<existing kind>, payload=<existing payload>)`; signature attached via T-1159 identity
- [ ] Receiver side: where agents call `event.collect` / `event.poll` with a kind filter, rewrite to `channel.subscribe(topic="broadcast:global")` with client-side filter on `msg_type`
- [ ] Topic `broadcast:global` auto-created on hub startup (idempotent `channel.create`) so agents can post without a prior bootstrap step
- [ ] Keep `event.broadcast` router method operational — it internally forwards to the channel.post path (shim). Add `#[deprecated(note = "migrate to channel.post topic=broadcast:global (T-1162)")]`
- [ ] Integration test: two test sessions, one broadcasts via legacy `event.broadcast`, other via new `channel.post` — both arrive at subscribers of `broadcast:global`
- [ ] Dispatch coordination path (T-914/T-916 lineage) still works end-to-end after the migration — `termlink dispatch-status` shows active workers, workers report back
- [ ] T-1071 "broadcast resilience" pattern preserved: fallback to `event.broadcast` if new topic doesn't exist (pre-hub-upgrade peers) — use the capabilities handshake already in place
- [ ] `cargo build && cargo test && cargo clippy -- -D warnings` pass workspace-wide
- [ ] Telemetry: `termlink fleet doctor` reports `broadcast.migration.status = partial | complete` based on the call-site audit

### Human
- [ ] [REVIEW] Smoke-test a real dispatch cycle after the migration
  **Steps:**
  1. Run `termlink dispatch "echo hello"` against a local hub
  2. Confirm the worker runs, reports back, and `event.collect` still returns the exit code
  3. Run `termlink channel subscribe broadcast:global` in one terminal while dispatching in another — verify events are visible under both old (`event.collect`) and new (`channel.subscribe`) APIs
  **Expected:** Both paths observe the same events
  **If not:** Note which direction leaks; open a follow-up

## Verification

cargo build
cargo test -p termlink-hub broadcast
cargo clippy -- -D warnings
grep -rn "event.broadcast\|event_broadcast" crates/ | tee /tmp/T-1162-callsites.txt
grep -q "broadcast:global" crates/termlink-hub/src/router.rs

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
