---
id: T-734
name: "Add ok:true to status and remote status JSON success responses"
description: >
  Add ok:true to status and remote status JSON success responses

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-cli/src/commands/remote.rs, 
      crates/termlink-cli/src/commands/session.rs]
related_tasks: []
created: 2026-03-29T12:03:35Z
last_update: '2026-08-18T18:59:20Z'
date_finished: 2026-03-29T12:04:38Z
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
      blast_radius: 3
      tier: 2
      effort: 3
    rationale: blast_radius=3 (no-signal); tier=2 (no-signal); effort=3 
      (no-signal)
    rubric_sha: missing
---

# T-734: Add ok:true to status and remote status JSON success responses

## Context

`status --json` and `remote status --json` pass through bare RPC results without `"ok": true`.

## Acceptance Criteria

### Agent
- [x] `status --json` success wraps result with `"ok": true`
- [x] `remote status --json` success wraps result with `"ok": true`
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

### 2026-03-29T12:03:35Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-734-add-oktrue-to-status-and-remote-status-j.md
- **Context:** Initial task creation

### 2026-03-29T12:04:38Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
