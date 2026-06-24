---
id: T-1471
name: "fleet doctor: configurable --top-callers <N> (per-hub + fleet-wide)"
description: >
  fleet doctor: configurable --top-callers <N> (per-hub + fleet-wide)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/remote.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-05-04T07:22:47Z
last_update: 2026-05-04T07:30:28Z
date_finished: 2026-05-04T07:30:28Z
---

# T-1471: fleet doctor: configurable --top-callers <N> (per-hub + fleet-wide)

## Context

`fleet doctor --legacy-usage` shows the top 3 callers per hub and the top 3
fleet-wide. With T-1467's `by_method` fallback enabled, the synthetic
`(unknown)` caller often dominates the list (currently 20,360 of 20,381
fleet residue calls = 99.9%), so 2 of the 3 visible slots are spent on
ranks 2 and 3 of identifiable callers. The fleet has 7+ identifiable
callers — the operator can't see them all without parsing JSON.

`--top-callers N` raises the cap. Default stays 3 (no behaviour change for
existing scripts/cron). Cap at 50 to prevent runaway output.

## Acceptance Criteria

### Agent
- [x] New flag `--top-callers <N>` added to `FleetAction::Doctor` with default 3, clamped at runtime to `1..=50`
- [x] Both per-hub display loop and fleet-wide aggregator respect the configured count
- [x] cmd_fleet_doctor signature threads the new arg through; main.rs dispatch updated
- [x] `cargo build --release -p termlink` succeeds
- [x] Live verification: `target/release/termlink fleet doctor --legacy-usage --top-callers 10` shows >3 callers per hub (where the data exists) AND a longer fleet-wide list

## Verification
cargo build --release -p termlink 2>&1 | tail -3

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

### 2026-05-04T07:22:47Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1471-fleet-doctor-configurable---top-callers-.md
- **Context:** Initial task creation

### 2026-05-04T07:27:55Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed

### 2026-05-04T07:30:28Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
