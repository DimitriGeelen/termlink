---
id: T-544
name: "Doctor version shows commit hash"
description: >
  Doctor version shows commit hash

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-cli/src/commands/infrastructure.rs]
related_tasks: []
created: 2026-03-28T09:25:45Z
last_update: '2026-08-18T18:59:17Z'
date_finished: 2026-03-28T09:26:38Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:01Z'
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
  - ts: '2026-08-18T18:59:17Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 2
      effort: 2
    rationale: blast_radius=1 (no-signal); tier=2 (no-signal); effort=2 
      (no-signal)
    rubric_sha: missing
---

# T-544: Doctor version shows commit hash

## Context

`termlink doctor` version check shows only version number, not the git commit hash. Add commit hash for debugging parity with `termlink version` and `termlink info`.

## Acceptance Criteria

### Agent
- [x] `termlink doctor` version check shows commit hash
- [x] `cargo build` succeeds

## Verification

cargo build 2>&1

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

### 2026-03-28T09:25:45Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-544-doctor-version-shows-commit-hash.md
- **Context:** Initial task creation

### 2026-03-28T09:26:38Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
