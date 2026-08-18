---
id: T-805
name: "Add since parameter to event.subscribe for cursor-based replay"
description: >
  Add since parameter to event.subscribe for cursor-based replay

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-30T17:37:40Z
last_update: '2026-08-18T18:59:21Z'
date_finished: 2026-03-30T17:42:13Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:10Z'
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
  - ts: '2026-08-18T18:59:21Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 8
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-805: Add since parameter to event.subscribe for cursor-based replay

## Context

`event.subscribe` uses a broadcast channel for push-based delivery but has no way to replay historical events. Clients that reconnect or start late miss events between their last `next_seq` and the subscribe call. `event.poll` already supports `since` for catch-up. Adding `since` to `event.subscribe` lets clients do catch-up + live delivery in one RPC call (poll buffer first, then stream live).

## Acceptance Criteria

### Agent
- [x] `event.subscribe` accepts optional `since` parameter (u64 sequence number)
- [x] When `since` is provided, historical events with seq > since are included before live events
- [x] Gap detection reports `gap_detected` and `events_lost` when since falls before oldest buffered event
- [x] Topic filter applies to both historical and live events
- [x] `max_events` limit applies across historical + live events combined
- [x] Without `since` parameter, behavior is unchanged (only live events)
- [x] Unit tests cover: since with history, since with gap, since with topic filter, since without matching events
- [x] `cargo check -p termlink` passes
- [x] `cargo test -p termlink-session` passes

## Verification

grep -q "since_param" crates/termlink-session/src/handler.rs
grep -c "event_subscribe_since" crates/termlink-session/src/handler.rs | grep -q "3"

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

### 2026-03-30T17:37:40Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-805-add-since-parameter-to-eventsubscribe-fo.md
- **Context:** Initial task creation

### 2026-03-30T17:42:13Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
