---
id: T-856
name: "Fix dispatch test ordering bug — mutual exclusion check must precede git repo
  check"
description: >
  Fix dispatch test ordering bug — mutual exclusion check must precede git repo check

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-cli/src/commands/dispatch.rs]
related_tasks: []
created: 2026-04-04T19:01:38Z
last_update: '2026-08-18T18:59:22Z'
date_finished: 2026-04-04T19:04:26Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:12Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 1
      D2: 1
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=1 (body:fix-without-learning); D2=1 (body:log-or-error-line); 
      D3=0 (no-signal); D4=0 (no-signal); F-RECALL=0 (no-signal); F-ORCH=0 
      (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:22Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 2
      effort: 3
    rationale: blast_radius=1 (no-signal); tier=2 (no-signal); effort=3 
      (no-signal)
    rubric_sha: missing
---

# T-856: Fix dispatch test ordering bug — mutual exclusion check must precede git repo check

## Context

`isolate_and_workdir_mutually_exclusive` test fails when run in full workspace because `isolate_rejects_non_git_dir` changes CWD to a non-git temp dir. The git repo check fires before the mutual exclusion check, producing wrong error message.

## Acceptance Criteria

### Agent
- [x] Mutual exclusion check (`--isolate` + `--workdir`) runs before git repository check in `cmd_dispatch`
- [x] `cargo test --workspace` passes with 0 failures (including the previously-flaky test)
- [x] Zero clippy warnings: `cargo clippy --workspace`

## Verification

cargo test --workspace
cargo clippy --workspace -- -D warnings

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

### 2026-04-04T19:01:38Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-856-fix-dispatch-test-ordering-bug--mutual-e.md
- **Context:** Initial task creation

### 2026-04-04T19:04:26Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
