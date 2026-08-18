---
id: T-035
name: "Control plane resize — command.resize RPC method and CLI command"
description: >
  Control plane resize — command.resize RPC method and CLI command

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-08T20:52:00Z
last_update: '2026-08-18T18:58:41Z'
date_finished: 2026-03-08T20:54:19Z
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
  - ts: '2026-08-18T18:58:41Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 7
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=7 
      (no-signal)
    rubric_sha: missing
---

# T-035: Control plane resize — command.resize RPC method and CLI command

## Context

Control plane resize: `command.resize` RPC method and `termlink resize` CLI command. Complements data plane Resize frames from T-032. 14 CLI commands total.

## Acceptance Criteria

### Agent
- [x] `COMMAND_RESIZE` constant added to protocol
- [x] `handle_command_resize` handler validates cols/rows, calls pty.resize()
- [x] Returns error for non-PTY sessions (CAPABILITY_NOT_SUPPORTED)
- [x] Returns error for missing/invalid params (INVALID_PARAMS)
- [x] `termlink resize <target> <cols> <rows>` CLI command
- [x] 2 new handler tests (no_pty, missing_params)
- [x] All 116 tests pass

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

### 2026-03-08T20:52:00Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-035-control-plane-resize--commandresize-rpc-.md
- **Context:** Initial task creation

### 2026-03-08T20:54:19Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
