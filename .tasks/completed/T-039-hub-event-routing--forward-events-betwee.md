---
id: T-039
name: "Hub event routing — forward events between sessions"
description: >
  Hub event routing — forward events between sessions

status: work-completed
workflow_type: build
owner: claude-code
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-08T21:15:36Z
last_update: '2026-08-18T18:58:41Z'
date_finished: 2026-03-08T21:20:44Z
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

# T-039: Hub event routing — forward events between sessions

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] Hub handles `event.broadcast` — fan-out emit to multiple sessions
- [x] Hub handles `event.collect` — fan-in poll from multiple sessions with cursor tracking
- [x] `event.broadcast` supports optional `targets` filter
- [x] `event.collect` supports `since` cursors and `topic` filter
- [x] CLI `broadcast` command added
- [x] 5 new hub tests (broadcast, broadcast-filtered, collect, collect-cursors, broadcast-error)
- [x] All 131 tests pass

## Verification

/Users/dimidev32/.cargo/bin/cargo build -p termlink-hub 2>&1 | tail -1 | grep -q "Finished"
grep -q "event.broadcast" crates/termlink-hub/src/router.rs
grep -q "event.collect" crates/termlink-hub/src/router.rs
grep -q "Broadcast" crates/termlink-cli/src/main.rs

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

### 2026-03-08T21:15:36Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-039-hub-event-routing--forward-events-betwee.md
- **Context:** Initial task creation

### 2026-03-08T21:20:44Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
