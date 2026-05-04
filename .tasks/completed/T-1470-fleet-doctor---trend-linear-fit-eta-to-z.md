---
id: T-1470
name: "fleet doctor --trend: linear-fit ETA-to-zero forecast"
description: >
  fleet doctor --trend: linear-fit ETA-to-zero forecast

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/remote.rs]
related_tasks: []
created: 2026-05-04T07:08:14Z
last_update: 2026-05-04T07:21:29Z
date_finished: 2026-05-04T07:21:29Z
---

# T-1470: fleet doctor --trend: linear-fit ETA-to-zero forecast

## Context

T-1468's trend block tells the operator the trajectory (`decreasing`/`flat`/
`increasing`) but stops short of *when* zero is reached. For cut-decision
planning the operator wants:
"trajectory: decreasing — estimated zero: 2026-05-23 (19 days at current rate)".

This task fits a least-squares line through the trend points (each point's
ts_ms is x, total is y), extrapolates to y=0, and renders the resulting
date plus days-from-today. Falls back gracefully when the fit is meaningless
(<2 points, slope ≥ 0, or any ts_ms missing — pre-T-1463 snapshots).

Schema-additive: adds an optional `eta_zero` block to `legacy_summary.trend`
in JSON mode (`{date_iso, days_from_now, slope_per_day}` or omitted/null
when the forecast doesn't apply). Non-JSON mode adds one line under the
trajectory verdict.

## Acceptance Criteria

### Agent
- [x] Pure helper `compute_eta_to_zero(points: &[TrendPoint], now_ms: u64) -> Option<EtaForecast>` in `commands/remote.rs` — returns `None` when forecast doesn't apply (see below)
- [x] `EtaForecast` struct: `{slope_per_day: f64, days_from_now: f64, target_ms: u64}`
- [x] Forecast returns `None` when: fewer than 2 points with `ts_ms`, all points have the same total (zero slope), slope ≥ 0 (flat or growing), or current total ≤ 0
- [x] Linear fit uses least-squares regression on (ts_ms, total) pairs (operates on f64 internally)
- [x] Human-mode trend block adds a line: `  ETA to zero: 2026-05-23 (19.4 days at -1037/day)` when forecast applies; suppressed otherwise
- [x] JSON-mode `legacy_summary.trend` gains an `eta_zero` field (object or null)
- [x] At least 5 unit tests: <2 points returns None; flat returns None; growing returns None; clean linear decay produces correct ETA; noisy decay (with one inflection point) produces a sensible forward date
- [x] `cargo build --release -p termlink` succeeds
- [x] `cargo test --release -p termlink --bins eta_to_zero` passes
- [x] Live verification: against the T-1468 fixture (3 snapshots: 25k → 24.5k → 22k → 20.4k current), trend block prints both trajectory + ETA line

## Verification
cargo build --release -p termlink 2>&1 | tail -3
cargo test --release -p termlink --bins eta_to_zero 2>&1 | tail -3

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

### 2026-05-04T07:08:14Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1470-fleet-doctor---trend-linear-fit-eta-to-z.md
- **Context:** Initial task creation

### 2026-05-04T07:21:29Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
