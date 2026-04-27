---
id: T-1309
name: "Enrich rpc-audit with caller attribution (from field) — T-1166 hunt-down enabler"
description: >
  Enrich rpc-audit with caller attribution (from field) — T-1166 hunt-down enabler

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [T-1166, T-1304-followup, hub, telemetry, observability]
components: [crates/termlink-hub/src/rpc_audit.rs, crates/termlink-hub/src/server.rs, .agentic-framework/agents/metrics/api-usage.sh]
related_tasks: [T-1304, T-1307, T-1308, T-1166]
created: 2026-04-27T12:26:32Z
last_update: 2026-04-27T12:26:32Z
date_finished: null
---

# T-1309: Enrich rpc-audit with caller attribution (from field) — T-1166 hunt-down enabler

## Context

T-1304 ships per-method counts. T-1308 ships trend windows. Operators driving the T-1166 retirement now see *how much* legacy traffic remains across each window — but not *who* is generating it. Today's hunt path: grep audit log for `event.broadcast`, manually correlate timestamps against session logs to guess at the caller. Doesn't scale across a fleet.

Enrich `rpc-audit.jsonl` with the optional `from` field that callers already supply on `event.broadcast` / `event.emit_to` (T-1300 plumbing). Audit line becomes:

```
{"ts":1714234567890,"method":"event.broadcast","from":"framework-agent"}
```

`from` is omitted when the caller did not provide it (legacy clients, methods that don't carry the field). Backwards-compatible — existing readers see an extra optional field, missing field reads as None/null.

`fw metrics api-usage` then groups legacy methods by `from` so operators see:

```
Legacy callers (last 7d):
   23  event.broadcast  framework-agent
    4  inbox.list        ring20-management/claude-dev
    1  event.broadcast  (unknown)
```

That's the missing dimension — converts "we have 0.5% legacy" into "framework-agent is responsible for 23 of 28 calls; migrate it next."

Pure additive. No legacy primitive is changed, no caller is forced to upgrade. The audit log already preserves what was asked of the hub, this just preserves *who* asked.

## Acceptance Criteria

### Agent
- [x] `rpc_audit::record(method, from)` accepts an optional caller string; omits `from` from the JSON line when None
- [x] Existing call site in `server.rs` extracts optional `from` from `req.params` (top-level field) and threads it through; non-string values treated as None
- [x] Audit line shape: `{"ts":...,"method":"...","from":"..."}` when present, `{"ts":...,"method":"..."}` when absent — both must remain valid JSON parseable by the existing Python tally script
- [x] `fw metrics api-usage` shows legacy callers grouped by `from`, with `(unknown)` for entries missing the field — both trend mode (uses 60d window) and `--last-Nd N` single-window mode
- [x] At least 3 unit tests in `rpc_audit.rs`: (1) record with from writes the field, (2) record without from omits the field, (3) skip-list still applies
- [x] All existing rpc_audit tests still pass
- [x] `cargo build -p termlink-hub` clean, `cargo clippy -p termlink-hub --tests -- -D warnings` clean
- [x] `docs/operations/api-usage-metrics.md` line-format section updated to document the new optional field

## Verification

cargo build -p termlink-hub 2>&1 | tail -3 | grep -qE "Finished"
cargo test -p termlink-hub rpc_audit 2>&1 | tail -10 | grep -qE "test result: ok"
cargo test -p termlink-hub 2>&1 | tail -25 | grep -qE "test result: ok\.\s+[0-9]+ passed"
cargo clippy -p termlink-hub --tests -- -D warnings 2>&1 | tail -3 | grep -qE "Finished"
grep -q '"from"' crates/termlink-hub/src/rpc_audit.rs
grep -q "Legacy callers" .agentic-framework/agents/metrics/api-usage.sh
grep -q '"from"' docs/operations/api-usage-metrics.md

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

### 2026-04-27T12:26:32Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1309-enrich-rpc-audit-with-caller-attribution.md
- **Context:** Initial task creation

### 2026-04-27T12:42Z — build delivered [agent autonomous pass]
- **rpc_audit:** `record(method, from)` signature change. Legacy line shape preserved when from absent (back-compat); new shape `{"ts":...,"method":...,"from":"<display_name>"}` when present. Empty-string from treated as absent.
- **server.rs:** Dispatch entry point extracts `req.params.from` (string-only, anything else treated as None) and threads through to `rpc_audit::record`.
- **api-usage.sh:** parses optional `from` per line, builds `legacy_callers` Counter keyed by `(method, from)`, renders "Legacy callers (last Nd):" block in both trend mode (60d window) and `--last-Nd N` single-window mode. `(unknown)` placeholder for entries without from.
- **Tests:** 3 new unit tests in rpc_audit (`line_with_from_includes_field`, `line_without_from_omits_field`, `empty_from_treated_as_absent`); existing 6 tests updated for new signature. 9/9 green.
- **Verification:** synthetic mixed-format input (with/without from) renders correctly in both modes; gate exit codes preserved. `cargo test -p termlink-hub` 269/269 ok; clippy clean.
- All Agent ACs ticked.
