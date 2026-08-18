---
id: T-200
name: "Fix all clippy warnings across workspace"
description: >
  Fix all clippy warnings across workspace

status: work-completed
workflow_type: refactor
owner: agent
horizon:
tags: [quality, clippy]
components: []
related_tasks: []
created: 2026-03-20T23:36:53Z
last_update: '2026-08-18T18:59:00Z'
date_finished: 2026-03-20T23:47:50Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:22Z'
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
  - ts: '2026-08-18T18:59:00Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 3
      effort: 6
    rationale: blast_radius=0 (no-signal); tier=3 (no-signal); effort=6 
      (no-signal)
    rubric_sha: missing
---

# T-200: Fix all clippy warnings across workspace

## Context

59 clippy warnings across 4 crates. 47 auto-fixable (collapsed ifs, simplified map_or, borrow cleanup), 10 structural (missing is_empty, complex types, too_many_arguments).

## Acceptance Criteria

### Agent
- [x] `cargo clippy --workspace` produces 0 warnings
- [x] `RemoteStore` has `is_empty()` method (clippy::len_without_is_empty)
- [x] Complex types in `transport.rs` factored into `ConnectFuture`/`BindFuture` type aliases
- [x] `cargo test --workspace` passes with 0 failures (297 passed)
- [x] Auto-fixed: 47 warnings (collapsed if statements, simplified map_or, removed unnecessary borrows/derefs, div_ceil, Error::other)
- [x] Manual fixes: 5 `too_many_arguments` suppressed with targeted `#[allow]`, 1 `is_empty` added, 3 complex types aliased

## Verification

/Users/dimidev32/.cargo/bin/cargo build --workspace
/Users/dimidev32/.cargo/bin/cargo test --workspace 2>&1 | grep -v "^$" | tail -1 | grep -q "0 failed"

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

### 2026-03-20T23:36:53Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-200-fix-all-clippy-warnings-across-workspace.md
- **Context:** Initial task creation

### 2026-03-20T23:47:50Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
