---
id: T-226
name: "Fix deprecated cargo_bin import in CLI integration tests"
description: >
  Fix deprecated cargo_bin import in CLI integration tests

status: work-completed
workflow_type: refactor
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-21T10:38:47Z
last_update: '2026-08-18T18:59:06Z'
date_finished: 2026-03-21T10:41:40Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:36Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=0 (no-signal); D4=0 
      (no-signal); F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:06Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 3
      effort: 2
    rationale: blast_radius=0 (no-signal); tier=3 (no-signal); effort=2 
      (no-signal)
    rubric_sha: missing
---

# T-226: Fix deprecated cargo_bin import in CLI integration tests

## Context

`cargo test` emits a deprecation warning: `use of deprecated function assert_cmd::cargo::cargo_bin`. The `cargo_bin!` macro is used but the function import triggers the warning.

## Acceptance Criteria

### Agent
- [x] Deprecation warning eliminated from `cargo test -p termlink` output
- [x] All 15 integration tests still pass

## Verification

! /Users/dimidev32/.cargo/bin/cargo test -p termlink --test cli_integration --no-run --manifest-path /Users/dimidev32/001-projects/010-termlink/Cargo.toml 2>&1 | grep -q 'deprecated'

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

### 2026-03-21T10:38:47Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-226-fix-deprecated-cargobin-import-in-cli-in.md
- **Context:** Initial task creation

### 2026-03-21T10:41:40Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
