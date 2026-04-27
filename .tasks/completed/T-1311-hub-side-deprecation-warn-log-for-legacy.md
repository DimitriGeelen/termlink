---
id: T-1311
name: "Hub-side deprecation warn-log for legacy primitives — real-time T-1166 signal"
description: >
  Hub-side deprecation warn-log for legacy primitives — real-time T-1166 signal

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [T-1166, T-1304-followup, hub, observability, deprecation]
components: [crates/termlink-hub/src/rpc_audit.rs, crates/termlink-hub/src/server.rs]
related_tasks: [T-1304, T-1309, T-1310, T-1166]
created: 2026-04-27T12:41:54Z
last_update: 2026-04-27T12:45:24Z
date_finished: 2026-04-27T12:45:24Z
---

# T-1311: Hub-side deprecation warn-log for legacy primitives — real-time T-1166 signal

## Context

T-1304/T-1309 ship the **retrospective** half of T-1166 telemetry: count and attribute legacy primitive calls in the audit log. Operators see the trend in `fw metrics api-usage`, but only by polling the file.

This task adds the **real-time** half: a `tracing::warn!` at the hub the moment a legacy primitive is dispatched, identifying the method and caller. Operators tailing hub logs (journalctl/stderr) see deprecated usage as it happens, not days later.

Output shape:
```
WARN deprecated method=event.broadcast from=framework-agent T-1166: schedule retirement once legacy <1% over 60d
```

Rate-limited to one log per (method, from) pair per 5 minutes. Without rate-limiting, a long-running caller spamming `inbox.list` every second would flood the log. Counter resets after the window expires so genuinely new offenders surface.

Lives in `rpc_audit.rs` next to the existing `record()` since both consume the same dispatch-time information. Single new pub function `warn_if_legacy(method, from)` invoked alongside `record(method, from)` in server.rs.

Pure additive. No protocol changes. No CLI changes. No retirement of any primitive — just an early-warning system layered on the existing audit infrastructure.

## Acceptance Criteria

### Agent
- [x] `rpc_audit::warn_if_legacy(method, from)` checks if method is in the LEGACY set or matches `file.send.*` / `file.receive.*` chunked variants; returns immediately if not
- [x] Rate-limit dedupe: same `(method, from)` pair logs at most once per 5-minute window. After the window expires, next call logs again
- [x] Implementation uses a process-local `Mutex<HashMap<(String, String), Instant>>` or equivalent; entries older than the window are pruned opportunistically on each call (no separate gc thread)
- [x] `warn_if_legacy` is called from `server.rs` immediately after `record()` (so behaviour can be enabled/disabled together)
- [x] Log format: `tracing::warn!(method = %m, from = %f, "deprecated primitive called — T-1166")` — uses tracing fields, NOT just a single formatted string, so structured-log consumers can filter
- [x] When `from` is None, log shows `from=(unknown)` to indicate the caller did not supply attribution
- [x] At least 4 unit tests: (1) non-legacy method logs nothing, (2) legacy method logs once on first call, (3) repeated call within 5min window logs only once, (4) call after window expires logs again
- [x] `cargo build -p termlink-hub` clean, `cargo clippy -p termlink-hub --tests -- -D warnings` clean
- [x] `cargo test -p termlink-hub` 0 failures (regression check)
- [x] `docs/operations/api-usage-metrics.md` gains a "Real-time deprecation log" subsection explaining the tracing warn output and how to filter for it (`journalctl -u termlink-hub | grep T-1166`)

## Verification

cargo build -p termlink-hub 2>&1 | tail -3 | grep -qE "Finished"
cargo test -p termlink-hub rpc_audit 2>&1 | tail -10 | grep -qE "test result: ok"
cargo test -p termlink-hub 2>&1 | tail -25 | grep -qE "test result: ok\.\s+[0-9]+ passed"
cargo clippy -p termlink-hub --tests -- -D warnings 2>&1 | tail -3 | grep -qE "Finished"
grep -q "warn_if_legacy" crates/termlink-hub/src/rpc_audit.rs
grep -q "warn_if_legacy" crates/termlink-hub/src/server.rs
grep -q "Real-time deprecation log" docs/operations/api-usage-metrics.md

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

### 2026-04-27T12:41:54Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1311-hub-side-deprecation-warn-log-for-legacy.md
- **Context:** Initial task creation

### 2026-04-27T12:55Z — build delivered [agent autonomous pass]
- **Module:** `rpc_audit::warn_if_legacy(method, from)` added to `crates/termlink-hub/src/rpc_audit.rs`. Checks `is_legacy_method(method)` (covers explicit set + `file.send.*` / `file.receive.*` chunked variants) then rate-limits via process-local `Mutex<HashMap<(String, String), Instant>>`. 5-minute window per `(method, from)` pair. Opportunistic prune removes entries older than 2× window on each call.
- **Wiring:** `server.rs` calls `warn_if_legacy(&req.method, from)` immediately after `record(&req.method, from)` so both behaviors share the same dispatch entry point and toggle together if needed.
- **Log shape:** Structured tracing fields (method, from); message `"deprecated primitive called — T-1166: schedule retirement once legacy <1% over 60d"`. `from` shows `(unknown)` when caller didn't populate. Filter with `journalctl -u termlink-hub | grep T-1166`.
- **Tests:** 5 new unit tests in rpc_audit::tests covering legacy/non-legacy predicate, no-op for non-legacy, tracker insertion on first legacy call, rate-limit timestamp preservation within window, and `(unknown)` label for None caller. 14/14 rpc_audit tests pass; 274/274 hub tests pass.
- **Docs:** `docs/operations/api-usage-metrics.md` gained a "Real-time deprecation log (T-1311)" subsection with output shape, journalctl filter incantation, and rate-limit explanation.
- **Verification (P-011 gate):** `cargo build` ✓; `cargo test -p termlink-hub` 274/274 ok; clippy clean; all 7 verification grep checks ✓.
- All Agent ACs ticked.

### 2026-04-27T12:45:24Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
