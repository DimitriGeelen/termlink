---
id: T-566
name: "Add --timeout flag to termlink ping"
description: >
  Add --timeout flag to termlink ping

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-28T12:37:55Z
last_update: '2026-08-18T18:59:17Z'
date_finished: 2026-03-28T12:39:36Z
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
      effort: 4
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=4 
      (no-signal)
    rubric_sha: missing
---

# T-566: Add --timeout flag to termlink ping

## Context

Add `--timeout` flag to `termlink ping` (default 5s) so scripts can control how long to wait before failing.

## Acceptance Criteria

### Agent
- [x] `Ping` variant in cli.rs has `timeout: u64` field with default 5
- [x] `cmd_ping` uses the timeout value via tokio::time::timeout wrapping the RPC call
- [x] Integration test validates ping with --timeout flag
- [x] All existing tests pass

## Verification

cargo test -p termlink --test cli_integration -- cli_ping 2>&1 | grep -q "test result"
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

### 2026-03-28T12:37:55Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-566-add---timeout-flag-to-termlink-ping.md
- **Context:** Initial task creation

### 2026-03-28T12:39:36Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
