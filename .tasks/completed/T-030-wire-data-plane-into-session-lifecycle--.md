---
id: T-030
name: "Wire data plane into session lifecycle — shell sessions auto-start data server"
description: >
  Wire data plane into session lifecycle — shell sessions auto-start data server

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-08T20:29:18Z
last_update: '2026-08-18T18:58:40Z'
date_finished: 2026-03-08T20:31:06Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:39Z'
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
  - ts: '2026-08-18T18:58:40Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 5
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=5 
      (no-signal)
    rubric_sha: missing
---

# T-030: Wire data plane into session lifecycle — shell sessions auto-start data server

## Context

Wire the data plane (T-029) into the session lifecycle so `termlink register --shell` auto-starts a data server alongside the control plane. Predecessor: T-029 (data plane codec + streaming server).

## Acceptance Criteria

### Agent
- [x] `cmd_register --shell` creates broadcast channel and starts data server
- [x] PTY read loop uses `read_loop_with_broadcast(Some(tx))` instead of `read_loop()`
- [x] Data socket path printed during registration
- [x] Data socket cleaned up on Ctrl+C shutdown
- [x] All existing tests pass (110+)

## Verification

/Users/dimidev32/.cargo/bin/cargo test --workspace 2>&1 | tail -5

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

### 2026-03-08T20:29:18Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-030-wire-data-plane-into-session-lifecycle--.md
- **Context:** Initial task creation

### 2026-03-08T20:31:06Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
