---
id: T-033
name: "Report data plane availability in session status and discovery"
description: >
  Report data plane availability in session status and discovery

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-08T20:36:38Z
last_update: '2026-08-18T18:58:41Z'
date_finished: 2026-03-08T20:40:10Z
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
      effort: 6
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=6 
      (no-signal)
    rubric_sha: missing
---

# T-033: Report data plane availability in session status and discovery

## Context

Report data plane availability in session status, discovery, and registration metadata. Sessions started with `--shell` now advertise `data_plane` and `stream` capabilities.

## Acceptance Criteria

### Agent
- [x] `--shell` sessions add `data_plane` and `stream` to capabilities list
- [x] `data_socket` field added to SessionMetadata and persisted in registration JSON
- [x] `query.status` response includes `capabilities` array
- [x] `cmd_status` displays capabilities and data plane socket path
- [x] `persist_registration()` method added to Session for post-registration updates
- [x] All 110 tests pass

## Verification

/Users/dimidev32/.cargo/bin/cargo test --workspace 2>&1 | tail -5

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

### 2026-03-08T20:36:38Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-033-report-data-plane-availability-in-sessio.md
- **Context:** Initial task creation

### 2026-03-08T20:40:10Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
