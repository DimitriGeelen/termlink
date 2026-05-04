---
id: T-1468
name: "fleet doctor --legacy-usage --trend <dir>: multi-snapshot decay sparkline"
description: >
  fleet doctor --legacy-usage --trend <dir>: multi-snapshot decay sparkline

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-04T06:49:43Z
last_update: 2026-05-04T06:49:43Z
date_finished: null
---

# T-1468: fleet doctor --legacy-usage --trend <dir>: multi-snapshot decay sparkline

## Context

T-1462 (`--diff`) answers "what changed since one specific prior snapshot?"
The T-1466 cron wrapper writes a snapshot per day (`YYYY-MM-DD.json`), so
after a week of running there are 7+ snapshot files but no easy way to read
the rate trend without manually pairing them.

`--trend <dir>` reads the N most-recent snapshots in that directory (sorted
lexically/chronologically by filename), and prints a one-screen day-over-day
decay table: total fleet count per snapshot, delta from the prior, and a
short ASCII sparkline. Answers "is the residue actually decaying?" in one
command.

Schema-additive: emits a `trend` array under `legacy_summary` so dashboards
can consume the time-series.

## Acceptance Criteria

### Agent
- [x] New flag `--trend <PATH>` on `fleet doctor` (requires `--legacy-usage`; bails otherwise, matching `--diff`'s validation pattern)
- [x] New flag `--trend-keep <N>` to limit how many snapshots are read (default 7, max 30 — capped to avoid pathological dirs)
- [x] Pure helper `compute_legacy_trend(snapshots: &[(String, &serde_json::Value)]) -> Vec<TrendPoint>` — TrendPoint = (snapshot_label, ts_ms, total, delta_from_prior_signed)
- [x] Pure helper `render_sparkline(values: &[u64]) -> String` — Unicode block sparkline (▁▂▃▄▅▆▇█), normalized to max in series, returns empty string for empty/single-value input
- [x] Human-mode block prints: `Trend (last N snapshots):` table + sparkline + verdict-trajectory line ("decreasing / flat / increasing")
- [x] JSON-mode adds `trend: [...]` to `legacy_summary`
- [x] At least 6 unit tests covering: empty trend, single-snapshot trend, monotonic decrease (decreasing trajectory), monotonic increase, plateau (flat), sparkline normalization across uniform / single-spike / empty inputs
- [x] `cargo build --release -p termlink` succeeds
- [x] `cargo test --release -p termlink --bins legacy_trend` passes (≥6 tests)
- [x] Live verification: `target/release/termlink fleet doctor --legacy-usage --trend /tmp/trend-fixture/` produces the expected decreasing trajectory against a hand-built 3-snapshot fixture

## Verification
cargo build --release -p termlink 2>&1 | tail -3
cargo test --release -p termlink --bins legacy_trend 2>&1 | tail -5

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

### 2026-05-04T06:49:43Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1468-fleet-doctor---legacy-usage---trend-dir-.md
- **Context:** Initial task creation
