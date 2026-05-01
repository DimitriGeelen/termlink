---
id: T-1432
name: "fw fleet doctor --legacy-usage — telemetry for T-1166 cut readiness (T-1425 pick #6)"
description: >
  From T-1425 fast-forward synthesis. Walks each reachable hub's last-N-day event log (default 7d), counts inbox.push / file.send / event.broadcast invocations per session per host, renders as a table + summary in the doctor output. T-1166 cut readiness signal: when all reachable hubs report 0 legacy invocations for 7+ days, the cut is safe. Independent of every other pick — can ship anytime. Synergizes with T-1426 (deprecation print) but doesn't require it (counts are derivable from event log directly). Watchtower visualization is a nice-to-have follow-up, NOT in scope here — table output via doctor is sufficient for cut-readiness decision.

status: captured
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-01T07:02:56Z
last_update: 2026-05-01T07:02:56Z
date_finished: null
---

# T-1432: fw fleet doctor --legacy-usage — telemetry for T-1166 cut readiness (T-1425 pick #6)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [ ] `termlink fleet doctor --legacy-usage` parses correctly via `--help`
- [ ] `--legacy-usage` defaults to a 7-day lookback; `--legacy-window <days>` overrides (1–90 valid range)
- [ ] For each reachable hub in `~/.termlink/hubs.toml`, the doctor walks that hub's event log and counts invocations of: `inbox.push`, `inbox.pull`, `inbox.list`, `inbox.clear`, `file.send`, `file.receive`, `event.broadcast` — the full retirement set per T-1166
- [ ] Output table format: `<hub>  <session>  <method>  <count>  <last_seen_iso>` — sortable, parseable
- [ ] Summary line per hub: total legacy invocations + days-since-last-call (e.g. "ring20-management: 0 invocations, last 14d ago — CUT-READY")
- [ ] Fleet summary line: cut-readiness verdict ("CUT-READY: all hubs zero for 7+d" / "WAIT: <N> hubs with recent legacy traffic")
- [ ] Honors hubs that are unreachable — reports `<hub>: UNREACHABLE` rather than failing the whole doctor
- [ ] Implementation lives where existing `fleet doctor` lives (likely `crates/termlink-cli/src/commands/remote.rs::cmd_fleet_doctor` or sibling); a `--legacy-usage` flag adds the new section without changing default doctor output
- [ ] `--json` output: per-hub array with `{hub, sessions: [{session_id, methods: {<method>: {count, last_seen_iso}}}], summary: {total, days_since_last, cut_ready}}`
- [ ] Unit tests: parsing the event log into the count structure (mock fixture); rendering table from a known struct; cut-readiness verdict logic for {0-traffic, recent-traffic, unreachable-hub} cases
- [ ] No regressions in existing `fleet doctor` output without the flag

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

cargo build --release -p termlink 2>&1 | tail -5
cargo test --release -p termlink-cli --lib 2>&1 | grep -E "legacy_usage|cut_ready" | head -5
target/release/termlink fleet doctor --help 2>&1 | grep -q "legacy-usage"
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
