---
id: T-032
name: "Stream enhancements — terminal resize forwarding and scrollback catch-up"
description: >
  Stream enhancements — terminal resize forwarding and scrollback catch-up

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-08T20:34:47Z
last_update: '2026-08-18T18:58:41Z'
date_finished: 2026-03-08T20:36:27Z
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
      effort: 5
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=5 
      (no-signal)
    rubric_sha: missing
---

# T-032: Stream enhancements — terminal resize forwarding and scrollback catch-up

## Context

Make `termlink stream` production-quality: forward terminal resize (SIGWINCH → Resize frame) and fetch initial scrollback on connect so the user sees existing output. Predecessor: T-031.

## Acceptance Criteria

### Agent
- [x] SIGWINCH handler sends Resize frame with current terminal dimensions
- [x] Initial scrollback fetched via control plane on connect and printed before streaming
- [x] Resize payload uses big-endian u16 for cols/rows (matches data_server expectation)
- [x] All existing tests pass (110+)
- [x] Builds without warnings

## Verification

/Users/dimidev32/.cargo/bin/cargo build --workspace 2>&1 | grep -E "^(error|warning:)" | head -5; test $? -le 1
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

### 2026-03-08T20:34:47Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-032-stream-enhancements--terminal-resize-for.md
- **Context:** Initial task creation

### 2026-03-08T20:36:27Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
