---
id: T-624
name: "Add --payload-only flag to event watch for raw payload extraction"
description: >
  Add --payload-only flag to event watch for raw payload extraction

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-28T17:47:00Z
last_update: '2026-08-18T18:59:18Z'
date_finished: 2026-03-28T17:48:39Z
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

# T-624: Add --payload-only flag to event watch for raw payload extraction

## Context

`event poll` has `--payload-only` but `event watch` doesn't. Consistency for scripting.

## Acceptance Criteria

### Agent
- [x] `--payload-only` flag added to `EventCommand::Watch` and hidden `Command::Watch` in cli.rs
- [x] `cmd_watch` outputs only payload JSON when `payload_only` is true
- [x] main.rs wires payload_only through both dispatch paths
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

### 2026-03-28T17:47:00Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-624-add---payload-only-flag-to-event-watch-f.md
- **Context:** Initial task creation

### 2026-03-28T17:48:39Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
