---
id: T-735
name: "Add ok:true to event poll, emit, broadcast, emit-to JSON success responses"
description: >
  Add ok:true to event poll, emit, broadcast, emit-to JSON success responses

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-29T13:14:30Z
last_update: '2026-08-18T18:59:20Z'
date_finished: 2026-03-29T13:16:23Z
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
      blast_radius: 0
      tier: 2
      effort: 5
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=5 
      (no-signal)
    rubric_sha: missing
---

# T-735: Add ok:true to event poll, emit, broadcast, emit-to JSON success responses

## Context

Add consistent `"ok": true` to the JSON success paths of `cmd_events`, `cmd_emit`, `cmd_broadcast`, and `cmd_emit_to` in events.rs. These currently pass through raw RPC results without wrapping.

## Acceptance Criteria

### Agent
- [x] `cmd_events` JSON success path includes `"ok": true` in output
- [x] `cmd_emit` JSON success path includes `"ok": true` in output
- [x] `cmd_broadcast` JSON success path includes `"ok": true` in output
- [x] `cmd_emit_to` JSON success path includes `"ok": true` in output
- [x] Project compiles with `cargo build`

## Verification

cd /opt/termlink && cargo build --release 2>&1 | tail -3

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

### 2026-03-29T13:14:30Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-735-add-oktrue-to-event-poll-emit-broadcast-.md
- **Context:** Initial task creation

### 2026-03-29T13:16:23Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
