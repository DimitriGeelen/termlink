---
id: T-080
name: "Hub pidfile + start/stop/status commands"
description: >
  Pidfile lifecycle for hub daemon: write PID on start, validate liveness on re-start,
  remove on shutdown. CLI commands: hub start (--daemonize), hub stop, hub status.

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-10T22:10:36Z
last_update: '2026-08-18T18:58:41Z'
date_finished: 2026-03-10T22:14:58Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:41Z'
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

# T-080: Hub pidfile + start/stop/status commands

## Context

From T-066 inception (GO). See [docs/reports/T-066-hub-daemon-inception.md].

## Acceptance Criteria

### Agent
- [x] Pidfile module in termlink-hub with write/read/validate/remove functions
- [x] Hub writes pidfile on start, removes on shutdown
- [x] Hub detects and cleans stale pidfile on start (dead PID)
- [x] Hub refuses to start if another hub is already running (live PID in pidfile)
- [x] `termlink hub` subcommands: start (foreground), stop, status
- [x] Unit tests for pidfile lifecycle (11 tests)
- [x] All existing hub tests continue to pass (22 total)

## Verification

/Users/dimidev32/.cargo/bin/cargo test -p termlink-hub 2>&1 | tail -5
/Users/dimidev32/.cargo/bin/cargo test -p termlink --test '*' 2>&1 | tail -5 || true

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

### 2026-03-10T22:10:36Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-080-hub-pidfile--startstopstatus-commands.md
- **Context:** Initial task creation

### 2026-03-10T22:10:54Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-10T22:14:58Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
