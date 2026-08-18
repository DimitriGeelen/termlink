---
id: T-281
name: "session.exited lifecycle event — supervisor emits before cleanup"
description: >
  Hub supervisor emits session.exited event before removing dead sessions. Crash safety
  net for dispatch orchestration. ~80 LOC across supervisor.rs, router.rs, protocol.

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: [dispatch, lifecycle, T-280]
components: []
related_tasks: [T-280, T-257, T-256]
created: 2026-03-25T15:08:31Z
last_update: '2026-08-18T18:59:17Z'
date_finished: 2026-03-25T15:18:27Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:01Z'
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
      effort: 7
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=7 
      (no-signal)
    rubric_sha: missing
---

# T-281: session.exited lifecycle event — supervisor emits before cleanup

## Context

From T-280 inception (GO). When a session's PID dies, TermLink emits nothing — hub supervisor
silently removes dead sessions. This makes dispatch orchestration unreliable: crashed workers
produce no signal, and orchestrators wait forever. Fix: supervisor emits `session.exited` event
before cleanup, giving orchestrators a reliable crash-detection signal.

## Acceptance Criteria

### Agent
- [x] Hub supervisor emits `session.exited` event (with session_id, display_name, pid) before removing dead sessions
- [x] Event is broadcast to all live sessions via hub router (fan-out)
- [x] Event includes exit reason field distinguishing clean exit vs process death
- [x] Integration test: spawn session, kill its PID, verify `session.exited` event appears on another session's event bus
- [x] Integration test: verify no event emitted for sessions that are still alive
- [x] All existing tests pass (0 regressions)
- [x] `cargo test --workspace` passes with 0 warnings

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

### 2026-03-25T15:08:31Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-281-sessionexited-lifecycle-event--superviso.md
- **Context:** Initial task creation

### 2026-03-25T15:18:27Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
