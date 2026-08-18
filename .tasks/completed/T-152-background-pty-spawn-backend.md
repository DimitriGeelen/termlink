---
id: T-152
name: "Background PTY spawn backend"
description: >
  Implement spawn_via_background: setsid + termlink register --shell as daemon fallback

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-16T05:45:33Z
last_update: '2026-08-18T18:58:51Z'
date_finished: 2026-03-16T06:33:29Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:01Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 2
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 1
    rationale: D1=0 (no-signal); D2=2 (body:telemetry-or-audit-entry); D3=0 
      (no-signal); D4=0 (no-signal); F-RECALL=0 (no-signal); F-ORCH=1 
      (body:hand-wired-dispatch)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:51Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 6
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=6 
      (no-signal)
    rubric_sha: missing
---

# T-152: Background PTY spawn backend

## Context

Background PTY fallback for environments without tmux or Terminal.app.

## Acceptance Criteria

### Agent
- [x] `termlink spawn --backend background --name test --shell --wait` spawns and registers
- [x] `termlink pty inject` and `termlink pty output` work on background-spawned session
- [x] `termlink interact --json` returns structured output with exit_code 0
- [x] Session cleanup works (kill PID, TermLink auto-cleans stale registration)
- [x] All existing tests pass (264)
- [x] Cross-platform: setsid on Linux, plain sh fallback on macOS

## Verification

/Users/dimidev32/.cargo/bin/cargo test --workspace

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

### 2026-03-16T05:45:33Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-152-background-pty-spawn-backend.md
- **Context:** Initial task creation

### 2026-03-16T06:09:14Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-16T06:33:29Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
