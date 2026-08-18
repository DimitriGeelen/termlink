---
id: T-637
name: "Add JSON error output to session command session-not-found errors"
description: >
  Add JSON error output to session command session-not-found errors

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-28T18:11:59Z
last_update: '2026-08-18T18:59:18Z'
date_finished: 2026-03-28T18:14:36Z
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
      effort: 2
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=2 
      (no-signal)
    rubric_sha: missing
---

# T-637: Add JSON error output to session command session-not-found errors

## Context

Session commands (ping, status, exec, send, signal) use `.context()` for session lookup without JSON output.

## Acceptance Criteria

### Agent
- [x] All 5 session commands (ping, status, exec, send, signal) have JSON error output for session-not-found
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

### 2026-03-28T18:11:59Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-637-add-json-error-output-to-session-command.md
- **Context:** Initial task creation

### 2026-03-28T18:14:36Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
