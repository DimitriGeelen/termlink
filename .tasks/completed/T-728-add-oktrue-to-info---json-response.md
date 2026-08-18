---
id: T-728
name: "Add ok:true to info --json response"
description: >
  Add ok:true to info --json response

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-cli/src/commands/session.rs]
related_tasks: []
created: 2026-03-29T11:45:56Z
last_update: '2026-08-18T18:59:20Z'
date_finished: 2026-03-29T11:46:42Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:08Z'
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
  - ts: '2026-08-18T18:59:20Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 2
      effort: 2
    rationale: blast_radius=1 (no-signal); tier=2 (no-signal); effort=2 
      (no-signal)
    rubric_sha: missing
---

# T-728: Add ok:true to info --json response

## Context

`info --json` outputs runtime info without `"ok": true`.

## Acceptance Criteria

### Agent
- [x] `info --json` includes `"ok": true` in response
- [x] Project compiles with `cargo check`

## Verification

cargo check 2>&1 | grep -q 'Finished'

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

### 2026-03-29T11:45:56Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-728-add-oktrue-to-info---json-response.md
- **Context:** Initial task creation

### 2026-03-29T11:46:42Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
