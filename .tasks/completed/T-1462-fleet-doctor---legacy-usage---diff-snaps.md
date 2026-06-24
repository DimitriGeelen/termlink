---
id: T-1462
name: "fleet doctor --legacy-usage --diff <snapshot> for decay rate measurement"
description: >
  fleet doctor --legacy-usage --diff <snapshot> for decay rate measurement

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/remote.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-05-04T05:18:46Z
last_update: 2026-05-04T05:32:53Z
date_finished: 2026-05-04T05:32:53Z
---

# T-1462: fleet doctor --legacy-usage --diff <snapshot> for decay rate measurement

## Context

T-1459/1460/1461 shipped point-in-time cut-readiness telemetry. Operators can
now see *who* is producing legacy traffic (`top_callers`) and the verdict
(CUT-READY / CUT-READY-DECAYING / WAIT). Missing piece: **rate of decay over
time**. Today, to answer "is the residue actually clearing?" the operator has
to eyeball two manual runs side-by-side. This task adds `--diff <snapshot>`
which reads a previously-saved JSON output and prints per-hub + per-caller +
fleet deltas, plus an average-per-minute rate when the snapshot timestamp is
embedded.

This is CLI-only — no hub upgrade required. Schema-additive: existing
`--legacy-usage --json` output already contains everything needed; we just
add a way to compare two snapshots.

## Acceptance Criteria

### Agent
- [x] `--diff <path>` flag added to `fleet doctor`; only meaningful when combined with `--legacy-usage`. Conflict with neither error nor side-effect on prior behavior.
- [x] `cmd_fleet_doctor` reads `<path>` as JSON; if file missing or unparseable, prints clear error and exits non-zero (does not silently fall through to normal output).
- [x] After current snapshot is computed, prints a `=== T-1166 cut-readiness DIFF vs <basename> ===` block showing: fleet `total_legacy` delta, per-hub count delta (hubs that appear/vanish flagged), per-caller delta from `top_callers_fleet`, and an average rate (calls/min) when both snapshots have a `_snapshot_ts_ms` field embedded.
- [x] Snapshot writes (when `--json` is used) include `_snapshot_ts_ms` (millis since epoch) at the top level so future `--diff` runs can compute elapsed time. Schema-additive — older consumers ignore unknown field.
- [x] Pure helper `compute_legacy_diff(prior: &Value, current: &Value) -> LegacyDiff` extracted so unit tests cover: (a) clean-to-clean (no diff), (b) decay (count went down), (c) growth (count went up), (d) hub vanished, (e) hub appeared, (f) caller dominance shift.
- [x] At least 4 unit tests pass: `cargo test -p termlink --bin termlink legacy_diff` (8/8 ok)
- [x] `cargo build --release -p termlink` succeeds
- [x] `cargo check -p termlink` produces no new warnings introduced by this change
- [x] Integration smoke: write a minimal hand-crafted snapshot to `/tmp/t1462-snap.json`, run `--legacy-usage --diff /tmp/t1462-snap.json` against current fleet, verify the DIFF block prints expected delta direction (synthetic prior produced expected NEW/VANISHED transitions; 4 error paths all exit non-zero with clear messages)


## Verification

cargo test -p termlink --bin termlink legacy_diff
cargo build --release -p termlink
grep -q "compute_legacy_diff" crates/termlink-cli/src/commands/remote.rs
grep -q "_snapshot_ts_ms" crates/termlink-cli/src/commands/remote.rs
grep -q "diff: Option<std::path::PathBuf>" crates/termlink-cli/src/cli.rs

## RCA

<!-- REQUIRED for bug-class tasks (workflow_type=build with bug-tag, OR title matches
     fix/bug/rca/broken/crash/error/regression/fail/hotfix).
     Non-bug-class tasks may leave this section empty or remove it.

     For bug-class, fill in:
       **Symptom:** what was observed (the user-facing manifestation).
       **Root cause:** the specific structural/logical gap — not "the code was wrong".
       **Why structurally allowed:** what in the framework/code/tooling let this go undetected.
       **Prevention:** what catches the next instance (test/lint/gate/doc/learning) — distinct from the fix itself.

     The completion gate (T-1550, G-019) blocks --status work-completed when
     bug-class AND this section is empty/template-only. Use --skip-rca to bypass (logged).
-->

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

### 2026-05-04T05:18:46Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1462-fleet-doctor---legacy-usage---diff-snaps.md
- **Context:** Initial task creation

### 2026-05-04T05:32:53Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
