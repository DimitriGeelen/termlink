---
id: T-628
name: "Add --ids flag to list command for session ID output"
description: >
  Add --ids flag to list command for session ID output

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-28T17:55:21Z
last_update: '2026-08-18T18:59:18Z'
date_finished: 2026-03-28T17:57:20Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:04Z'
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
  - ts: '2026-08-18T18:59:18Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 4
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=4 
      (no-signal)
    rubric_sha: missing
---

# T-628: Add --ids flag to list command for session ID output

## Context

`list --names` outputs display names but there's no way to get just IDs. `discover` has `--id` already.

## Acceptance Criteria

### Agent
- [x] `--ids` flag added to `Command::List` in cli.rs
- [x] `cmd_list` outputs one session ID per line when `ids` is true
- [x] main.rs wires the ids parameter
- [x] `cargo check -p termlink` passes

## Verification

cargo check -p termlink 2>&1 | tail -1

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

### 2026-03-28T17:55:21Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-628-add---ids-flag-to-list-command-for-sessi.md
- **Context:** Initial task creation

### 2026-03-28T17:57:20Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
