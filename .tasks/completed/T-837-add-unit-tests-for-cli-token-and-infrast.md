---
id: T-837
name: "Add unit tests for CLI token and infrastructure commands"
description: >
  Add unit tests for CLI token and infrastructure commands

status: work-completed
workflow_type: test
owner: agent
horizon:
tags: []
components: [crates/termlink-cli/src/commands/execution.rs]
related_tasks: []
created: 2026-04-03T23:20:57Z
last_update: '2026-08-18T18:59:22Z'
date_finished: 2026-04-03T23:29:08Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:11Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 3
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=3 (body:test-or-audit-check); D2=0 (no-signal); D3=0 
      (no-signal); D4=0 (no-signal); F-RECALL=0 (no-signal); F-ORCH=0 
      (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:22Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 1
      effort: 4
    rationale: blast_radius=1 (no-signal); tier=1 (no-signal); effort=4 
      (no-signal)
    rubric_sha: missing
---

# T-837: Add unit tests for CLI token and infrastructure commands

## Context

Add unit tests for `build_spawn_shell_cmd` and `resolve_spawn_backend` in execution.rs — the largest untested pure functions in the CLI crate.

## Acceptance Criteria

### Agent
- [x] Unit tests added for `build_spawn_shell_cmd` covering: shell-only, with command, with roles/tags/cap
- [x] Unit tests added for `resolve_spawn_backend` non-Auto paths
- [x] `cargo test --workspace` passes
- [x] Total test count increases

## Verification

cargo test --workspace 2>&1 | tail -3
cargo test -p termlink -- execution::tests 2>&1 | grep "test result" | head -1

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

### 2026-04-03T23:20:57Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-837-add-unit-tests-for-cli-token-and-infrast.md
- **Context:** Initial task creation

### 2026-04-03T23:29:08Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
