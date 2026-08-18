---
id: T-817
name: "Upgrade file receive event.poll to event.subscribe"
description: >
  Upgrade file receive event.poll to event.subscribe

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-cli/src/commands/file.rs]
related_tasks: []
created: 2026-03-30T19:58:32Z
last_update: '2026-08-18T18:59:21Z'
date_finished: 2026-03-30T20:06:37Z
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
      blast_radius: 1
      tier: 2
      effort: 5
    rationale: blast_radius=1 (no-signal); tier=2 (no-signal); effort=5 
      (no-signal)
    rubric_sha: missing
---

# T-817: Upgrade file receive event.poll to event.subscribe

## Context

Last `event.poll` sleep loop in the CLI. File receive uses poll for initial historical fetch (catches seq 0 file events), then poll+sleep for new events. Upgrade subsequent polling to `event.subscribe` while keeping `event.poll` for first historical fetch.

## Acceptance Criteria

### Agent
- [x] `event.poll` replaced with `event.subscribe` for live event waiting (after first poll)
- [x] First poll kept as `event.poll` (catches seq 0 historical events)
- [x] `tokio::time::sleep(poll_interval)` removed
- [x] `cargo check -p termlink` passes
- [x] File transfer integration tests pass (684 workspace total)

## Verification

cargo check -p termlink 2>&1 | grep -q "Finished"
cargo test -p termlink file

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

### 2026-03-30T19:58:32Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-817-upgrade-file-receive-eventpoll-to-events.md
- **Context:** Initial task creation

### 2026-03-30T20:06:37Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
