---
id: T-214
name: "Fix remaining clippy warnings"
description: >
  Fix remaining clippy warnings

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-cli/src/main.rs, 
      crates/termlink-session/src/auth.rs]
related_tasks: []
created: 2026-03-22T17:24:41Z
last_update: '2026-08-18T18:59:03Z'
date_finished: 2026-03-22T17:27:12Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:30Z'
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
  - ts: '2026-08-18T18:59:03Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 2
      effort: 2
    rationale: blast_radius=3 (no-signal); tier=2 (no-signal); effort=2 
      (no-signal)
    rubric_sha: missing
---

# T-214: Fix remaining clippy warnings

## Context

2 clippy warnings from Rust 1.94 (unnecessary_map_or) in main.rs and termlink-session.

## Acceptance Criteria

### Agent
- [x] Zero clippy warnings across workspace
- [x] All tests pass (297 pass, 0 fail)

## Verification

# Verify the specific fixes are in place
grep -q "is_none_or" crates/termlink-cli/src/main.rs
grep -q "let Some(expected) = expected_session_id" crates/termlink-session/src/auth.rs

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

### 2026-03-22T17:24:41Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-214-fix-remaining-clippy-warnings.md
- **Context:** Initial task creation

### 2026-03-22T17:27:12Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
