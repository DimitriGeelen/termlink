---
id: T-048
name: "CLI collect command — fan-in events from multiple sessions via hub"
description: >
  CLI collect command — fan-in events from multiple sessions via hub

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-08T23:00:53Z
last_update: '2026-08-18T18:58:41Z'
date_finished: 2026-03-08T23:04:55Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:40Z'
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

# T-048: CLI collect command — fan-in events from multiple sessions via hub

## Context

CLI interface for the hub's `event.collect` RPC — fan-in events from all (or targeted) sessions through the hub with continuous polling and cursor tracking.

## Acceptance Criteria

### Agent
- [x] `termlink collect` polls hub for events from all sessions
- [x] `--targets` filters to specific sessions
- [x] `--topic` filters by event topic
- [x] `--count N` exits after N events
- [x] `--interval` controls poll frequency
- [x] Cursor tracking prevents duplicate events across polls
- [x] All tests pass

## Verification

/Users/dimidev32/.cargo/bin/cargo build -p termlink 2>&1 | tail -1
/Users/dimidev32/.cargo/bin/cargo test 2>&1 | grep -E "^test result:" | grep -v "0 passed" | head -4

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

### 2026-03-08T23:00:53Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-048-cli-collect-command--fan-in-events-from-.md
- **Context:** Initial task creation

### 2026-03-08T23:04:55Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
