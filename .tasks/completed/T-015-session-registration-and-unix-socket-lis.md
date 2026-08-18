---
id: T-015
name: "Session registration and Unix socket listener"
description: >
  Session registration and Unix socket listener

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-08T16:00:31Z
last_update: '2026-08-18T18:58:40Z'
date_finished: 2026-03-08T16:26:26Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:39Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 2
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=0 (no-signal); D4=0 
      (no-signal); F-RECALL=2 (body:lightly-promoted); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:40Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 7
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=7 
      (no-signal)
    rubric_sha: missing
---

# T-015: Session registration and Unix socket listener

## Context

Implements T-006 design: session registration (JSON sidecar files), Unix socket listener, deregistration, liveness checks, and session listing. See `docs/reports/T-006-session-identity-discovery.md`.

## Acceptance Criteria

### Agent
- [x] Registration struct with serde JSON serialization matching T-006 format
- [x] `register()` creates runtime dir, binds Unix socket, writes atomic JSON registration
- [x] `deregister()` removes socket + JSON files cleanly
- [x] `is_alive()` performs PID-based liveness check
- [x] `list_sessions()` scans directory, parses registrations, filters stale entries
- [x] All new code has tests
- [x] `cargo test --workspace` passes with no failures

## Verification

PATH="$HOME/.cargo/bin:$PATH" cargo test --workspace
PATH="$HOME/.cargo/bin:$PATH" cargo clippy --workspace -- -D warnings

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

### 2026-03-08T16:00:31Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-015-session-registration-and-unix-socket-lis.md
- **Context:** Initial task creation

### 2026-03-08T16:26:26Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
