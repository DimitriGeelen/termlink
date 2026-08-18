---
id: T-120
name: "EventBus gap detection + concurrent hub broadcast"
description: >
  Fix EventBus silent event loss (gap detection when cursor < oldest_seq) and
  make hub broadcast concurrent instead of sequential. From T-009 inception GO.
status: work-completed
workflow_type: build
owner: agent
horizon:
tags: [concurrency, eventbus, hub, backpressure]
components: []
related_tasks: [T-009]
created: 2026-03-12T20:17:23Z
last_update: '2026-08-18T18:58:45Z'
date_finished: 2026-03-12T21:29:40Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:49Z'
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
  - ts: '2026-08-18T18:58:45Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 6
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=6 
      (no-signal)
    rubric_sha: missing
---

# T-120: EventBus gap detection + concurrent hub broadcast

## Context

From T-009 inception (docs/reports/T-009-exploration.md). EventBus ring buffer silently
evicts oldest events on overflow — pollers get no notification. Hub broadcast iterates
sessions sequentially — one dead session stalls all.

## Acceptance Criteria

### Agent
- [x] EventBus detects gap when poller cursor < oldest sequence in buffer
- [x] Gap detection returns a warning/error in the events response (not silent)
- [x] Hub broadcast dispatches to sessions concurrently (tokio::spawn per target, not sequential loop)
- [x] Hub broadcast has per-target timeout (not relying on default socket timeout)
- [x] Existing event ordering tests still pass
- [x] New test: concurrent pollers on one session see all events without loss

## Verification

# Rust tests pass
/Users/dimidev32/.cargo/bin/cargo test -p termlink-session --lib 2>&1 | grep -q "test result: ok"
/Users/dimidev32/.cargo/bin/cargo test -p termlink-hub --lib 2>&1 | grep -q "test result: ok"

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

### 2026-03-12T20:17:23Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-120-eventbus-gap-detection--concurrent-hub-b.md
- **Context:** Initial task creation

### 2026-03-12T21:29:40Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
