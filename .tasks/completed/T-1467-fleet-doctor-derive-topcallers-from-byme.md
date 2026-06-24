---
id: T-1467
name: "fleet doctor: derive top_callers from by_method fallback (works against pre-T-1460 hubs)"
description: >
  fleet doctor: derive top_callers from by_method fallback (works against pre-T-1460 hubs)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-cli/src/commands/remote.rs]
related_tasks: []
created: 2026-05-04T06:35:58Z
last_update: 2026-05-04T06:43:50Z
date_finished: 2026-05-04T06:43:50Z
---

# T-1467: fleet doctor: derive top_callers from by_method fallback (works against pre-T-1460 hubs)

## Context

T-1460/T-1461 added per-hub `top_callers` and fleet-wide aggregation (`top_callers_fleet`)
to `fleet doctor --legacy-usage`. The aggregation is gated on the hub returning the
new `top_callers` field — pre-T-1460 hubs (the entire live fleet at 0.9.0) leave it
unset, and `top_callers_fleet` is empty in current snapshots even though every hub's
`by_method.<method>.callers` array carries the same caller→count data in the older
shape.

This task adds a fallback in the CLI: when a hub's `legacy_usage` block has no
`top_callers` field but does have populated `by_method.*.callers`, derive an
equivalent (id → summed-count) list per hub. The fleet-wide aggregator
(`aggregate_fleet_top_callers`) then has data to summarise.

Result: fleet-wide top-callers display works against the unmodified 0.9.0 fleet,
giving operators the "WHO is still hitting legacy?" answer without waiting on any
hub upgrade. Schema-additive — no behavior change for post-T-1460 hubs (they still
ship `top_callers` directly and the fallback never triggers).

## Acceptance Criteria

### Agent
- [x] Pure helper `derive_top_callers_from_by_method(by_method: &serde_json::Value) -> Vec<(String, u64)>` added to `crates/termlink-cli/src/commands/remote.rs`, sums counts across methods per `from`, sorts descending by count
- [x] `cmd_fleet_doctor` legacy_summary builder uses the helper as a fallback when `top_callers` is absent or empty
- [x] At least 4 unit tests cover: empty by_method, single method one caller, multiple methods overlapping callers, post-T-1460 path (top_callers present → fallback NOT triggered)
- [x] `cargo build --release -p termlink` succeeds
- [x] `cargo test -p termlink derive_top_callers` passes
- [x] Live verification: running `target/release/termlink fleet doctor --legacy-usage --json` against the live fleet returns a non-empty `top_callers_fleet` (currently empty)

## Verification
cargo build --release -p termlink 2>&1 | tail -3
cargo test --release -p termlink --bins derive_top_callers 2>&1 | tail -3

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

### 2026-05-04T06:35:58Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1467-fleet-doctor-derive-topcallers-from-byme.md
- **Context:** Initial task creation

### 2026-05-04T06:43:50Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
