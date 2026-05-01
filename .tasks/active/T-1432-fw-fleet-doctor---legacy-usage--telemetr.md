---
id: T-1432
name: "fw fleet doctor --legacy-usage — telemetry for T-1166 cut readiness (T-1425 pick #6)"
description: >
  From T-1425 fast-forward synthesis. Walks each reachable hub's last-N-day event log (default 7d), counts inbox.push / file.send / event.broadcast invocations per session per host, renders as a table + summary in the doctor output. T-1166 cut readiness signal: when all reachable hubs report 0 legacy invocations for 7+ days, the cut is safe. Independent of every other pick — can ship anytime. Synergizes with T-1426 (deprecation print) but doesn't require it (counts are derivable from event log directly). Watchtower visualization is a nice-to-have follow-up, NOT in scope here — table output via doctor is sufficient for cut-readiness decision.

status: started-work
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-01T07:02:56Z
last_update: 2026-05-01T07:38:43Z
date_finished: null
---

# T-1432: fw fleet doctor --legacy-usage — telemetry for T-1166 cut readiness (T-1425 pick #6)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] `termlink fleet doctor --legacy-usage` parses correctly via `--help`; flag is opt-in (default doctor output unchanged)
- [x] `--legacy-window-days N` overrides the 7-day lookback; clamped to [1, 90]
- [x] For each reachable hub, doctor calls a new `hub.legacy_usage` Tier-A RPC that reads local `<runtime_dir>/rpc-audit.jsonl`, filters by window, returns counts. Retirement set tracked is `is_legacy_method()` in `rpc_audit.rs` — single source of truth covering `event.broadcast`, `inbox.{list,status,clear}`, `file.send`, `file.receive` plus chunked `file.send.*` / `file.receive.*` variants
- [x] Per-hub summary: total legacy + per-method breakdown + per-caller (`from`) breakdown. The audit log records `from` not `session_id`, so caller granularity is by display label
- [x] Fleet verdict: `CUT-READY` (all reachable hubs `audit_present=true` AND `total_legacy=0`), `WAIT` (≥1 hub with recent legacy traffic), `UNCERTAIN` (≥1 hub `audit_unsupported` because pre-T-1432, OR `audit_present=false`)
- [x] Hubs unreachable in connectivity probe are excluded from the legacy verdict (no double-fail) — already shown as FAIL in the per-hub block above
- [x] Implementation in `crates/termlink-cli/src/commands/remote.rs::cmd_fleet_doctor`; flags wired via `cli.rs::FleetAction::Doctor` + `main.rs` dispatch; hub handler `router.rs::handle_hub_legacy_usage` → `rpc_audit::summarize_legacy_usage`
- [x] `--json` adds top-level `legacy_summary` key (window_days, verdict, total_legacy_fleet, hubs_clean[], hubs_with_traffic[], hubs_unsupported[], hubs_no_audit[]) and embeds `legacy_usage` per-hub. Omitted entirely when flag not passed
- [x] Unit tests in `rpc_audit::tests`: `summarize_lines_counts_only_legacy_within_window`, `summarize_lines_empty_audit_returns_zero`, `summarize_lines_handles_missing_from_field` — extracted internal `summarize_lines` helper so tests don't need to poke `AUDIT_PATH` (OnceLock-only-once)
- [x] No regressions in default `fleet doctor` output without the flag — verified end-to-end against the live fleet
- [x] Pre-T-1432 hubs (every reachable hub today) gracefully fall back to `audit_unsupported: true` with an upgrade hint, instead of failing the whole probe

### Human
- [ ] [REVIEW] Verify the cut-readiness signal is actionable
  **Steps:**
  1. `termlink fleet doctor --legacy-usage` — see baseline
  2. Trigger a deliberate `inbox.push` (or a `file send`) somewhere reachable, then re-run
  3. Confirm the count incremented and `days_since_last` reset to 0
  4. After ~7d of clean operation post-T-1426 ship: re-run, confirm the verdict flips to CUT-READY
  **Expected:** the doctor's verdict tracks reality; agent operators trust it enough to flip T-1166's `LEGACY_PRIMITIVES_ENABLED=false`
  **If not:** capture which fleet event the doctor missed and re-scope

## Verification

cargo build --release -p termlink 2>&1 | tail -3
cargo test --release -p termlink-hub --lib rpc_audit::tests::summarize 2>&1 | grep -q "test result: ok. 3 passed"
target/release/termlink fleet doctor --help 2>&1 | grep -q "legacy-usage"
target/release/termlink fleet doctor --help 2>&1 | grep -q "legacy-window-days"
target/release/termlink fleet doctor --legacy-usage --json 2>&1 | python3 -c "import sys, json; d = json.load(sys.stdin); assert 'legacy_summary' in d; assert d['legacy_summary']['verdict'] in ('CUT-READY', 'WAIT', 'UNCERTAIN')"
target/release/termlink fleet doctor --legacy-usage 2>&1 | grep -q "T-1166 cut-readiness"
target/release/termlink fleet doctor --legacy-usage --json 2>&1 | head -1 | grep -q "^{"

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

### 2026-05-01T07:02:56Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1432-fw-fleet-doctor---legacy-usage--telemetr.md
- **Context:** Initial task creation

### 2026-05-01T07:29:03Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
