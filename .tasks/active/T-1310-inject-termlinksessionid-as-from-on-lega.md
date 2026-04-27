---
id: T-1310
name: "Inject TERMLINK_SESSION_ID as from on legacy primitive calls — fully populate T-1309 breakdown"
description: >
  Inject TERMLINK_SESSION_ID as from on legacy primitive calls — fully populate T-1309 breakdown

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [T-1166, T-1309-followup, cli, hub, telemetry]
components: [crates/termlink-cli/src/commands/events.rs, crates/termlink-session/src/inbox_channel.rs]
related_tasks: [T-1300, T-1304, T-1309, T-1166]
created: 2026-04-27T12:35:48Z
last_update: 2026-04-27T12:35:48Z
date_finished: null
---

# T-1310: Inject TERMLINK_SESSION_ID as from on legacy primitive calls — fully populate T-1309 breakdown

## Context

T-1300 added `$TERMLINK_SESSION_ID → from` injection only for `event.broadcast`. T-1309 made the audit log preserve `from`, so the legacy callers breakdown in `fw metrics api-usage` automatically populates from any caller that includes `from` in params.

But the breakdown today only works for broadcast, because the other three legacy primitives still emit `from`-less RPCs:

- `event.emit_to` (CLI: events.rs::cmd_emit_to) — passes `from` only when explicitly given on CLI
- `inbox.status` (legacy fallback: inbox_channel.rs::call_legacy_inbox_status_via_client) — params = `json!({})`
- `inbox.list` (legacy fallback: inbox_channel.rs::call_legacy_inbox_list_via_client) — params has only `target`
- `inbox.clear` (legacy fallback: inbox_channel.rs::call_legacy_inbox_clear_via_client) — params has only scope

Mirror T-1300's pattern in all four call sites: when the caller did not explicitly set `from`, populate it from `$TERMLINK_SESSION_ID` env var. Empty/missing env var = no `from` (preserves current behavior; T-1309 surfaces as `(unknown)`).

`file.send` / `file.receive` are NOT in scope: ripgrep across `crates/` shows zero current call sites; those primitives are listed in T-1166 only as "any old binary still in the wild".

Pure additive. No params shape changes (just adds an optional field). No public API signature changes.

## Acceptance Criteria

### Agent
- [x] `cmd_emit_to` in `crates/termlink-cli/src/commands/events.rs` injects `from = $TERMLINK_SESSION_ID` when params has no `from` and the env var is non-empty (mirrors `cmd_broadcast` pattern)
- [x] `call_legacy_inbox_status_via_client` injects `from` from `$TERMLINK_SESSION_ID` into the legacy RPC params
- [x] `call_legacy_inbox_list_via_client` injects `from` similarly
- [x] `call_legacy_inbox_clear_via_client` injects `from` similarly
- [x] Shared helper `params_with_session_from(base: serde_json::Value) -> serde_json::Value` (or equivalent inline pattern) reads the env var once per call and is the only place env-var lookup happens — keeps the four call sites uniform
- [x] Empty / unset `$TERMLINK_SESSION_ID` leaves params unchanged (no `from` key added)
- [x] At least 1 unit test in `inbox_channel.rs` demonstrating the helper adds `from` when env var set and leaves params unchanged when empty
- [x] `cargo build -p termlink -p termlink-session -p termlink-hub` clean
- [x] `cargo clippy -p termlink-session -p termlink-hub -p termlink --tests -- -D warnings` clean
- [x] `cargo test -p termlink-session -p termlink-hub` 0 failures

## Verification

cargo build -p termlink -p termlink-session -p termlink-hub 2>&1 | tail -3 | grep -qE "Finished"
cargo test -p termlink-session 2>&1 | tail -25 | grep -qE "test result: ok\.\s+[0-9]+ passed"
cargo test -p termlink-hub 2>&1 | tail -25 | grep -qE "test result: ok\.\s+[0-9]+ passed"
cargo clippy -p termlink-session -p termlink-hub -p termlink --tests -- -D warnings 2>&1 | tail -3 | grep -qE "Finished"
grep -q "TERMLINK_SESSION_ID" crates/termlink-session/src/inbox_channel.rs

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

### 2026-04-27T12:35:48Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1310-inject-termlinksessionid-as-from-on-lega.md
- **Context:** Initial task creation

### 2026-04-27T12:48Z — build delivered [agent autonomous pass]
- **Helper:** `params_with_session_from(Value) -> Value` in `crates/termlink-session/src/inbox_channel.rs` reads `$TERMLINK_SESSION_ID` once per call and injects `from` into params when caller has not set it. Empty/unset env var is a no-op (params unchanged). Explicit caller-provided `from` is preserved.
- **Wiring:** Threaded into all 3 legacy inbox call sites (`call_legacy_inbox_status_via_client`, `call_legacy_inbox_list_via_client`, `call_legacy_inbox_clear_via_client`) and mirrored in `cmd_emit_to` in `crates/termlink-cli/src/commands/events.rs` (matches the T-1300 broadcast pattern).
- **Tests:** Single sequential `params_with_session_from_all_scenarios` test in `inbox_channel::tests` covers 4 scenarios (unset / empty / injected / preserves explicit). Single test rather than 4 because std::env mutation is process-global and `cargo test` runs threads concurrently.
- **Verification (P-011 gate):** `cargo build` ✓; `cargo test -p termlink-session` 312/312 ok; `cargo test -p termlink-hub` 269/269 ok; `cargo clippy -p termlink-session -p termlink-hub -p termlink --tests -- -D warnings` ✓.
- All Agent ACs ticked. Together with T-1309, the audit log now captures caller attribution for all 4 legacy primitives that are reachable from the current CLI/MCP code paths.
