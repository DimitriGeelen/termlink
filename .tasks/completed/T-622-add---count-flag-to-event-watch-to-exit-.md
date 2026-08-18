---
id: T-622
name: "Add --count flag to event watch to exit after N events"
description: >
  Add --count flag to event watch to exit after N events

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-28T17:43:27Z
last_update: '2026-08-18T18:59:18Z'
date_finished: 2026-03-28T17:45:11Z
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
      effort: 5
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=5 
      (no-signal)
    rubric_sha: missing
---

# T-622: Add --count flag to event watch to exit after N events

## Context

`event collect` has `--count` but `event watch` doesn't, inconsistent for scripting.

## Acceptance Criteria

### Agent
- [x] `--count N` flag added to `EventCommand::Watch` and hidden `Command::Watch` in cli.rs
- [x] `cmd_watch` exits after receiving N events when count > 0
- [x] `--count 0` (default) means no limit (existing behavior preserved)
- [x] main.rs wires count through both dispatch paths
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

### 2026-03-28T17:43:27Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-622-add---count-flag-to-event-watch-to-exit-.md
- **Context:** Initial task creation

### 2026-03-28T17:45:11Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
