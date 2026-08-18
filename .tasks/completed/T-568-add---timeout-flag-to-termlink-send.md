---
id: T-568
name: "Add --timeout flag to termlink send"
description: >
  Add --timeout flag to termlink send

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-28T12:41:28Z
last_update: '2026-08-18T18:59:17Z'
date_finished: 2026-03-28T12:42:29Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:02Z'
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
      blast_radius: 0
      tier: 2
      effort: 3
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=3 
      (no-signal)
    rubric_sha: missing
---

# T-568: Add --timeout flag to termlink send

## Context

Add `--timeout` flag to `termlink send` (default 10s) for consistent timeout handling across RPC commands.

## Acceptance Criteria

### Agent
- [x] `Send` variant in cli.rs has `timeout: u64` field with default 10
- [x] `cmd_send` wraps RPC call in tokio::time::timeout
- [x] All existing tests pass

## Verification

cargo clippy -p termlink -- -D warnings 2>&1 | tail -1 | grep -qv error

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

### 2026-03-28T12:41:28Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-568-add---timeout-flag-to-termlink-send.md
- **Context:** Initial task creation

### 2026-03-28T12:42:29Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
