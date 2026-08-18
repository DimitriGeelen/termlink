---
id: T-050
name: "CLI info command — runtime diagnostics and system overview"
description: >
  CLI info command — runtime diagnostics and system overview

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-08T23:15:38Z
last_update: '2026-08-18T18:58:41Z'
date_finished: 2026-03-08T23:17:45Z
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
      effort: 5
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=5 
      (no-signal)
    rubric_sha: missing
---

# T-050: CLI info command — runtime diagnostics and system overview

## Context

Quick health check for the TermLink system — shows runtime paths, hub status, session counts.

## Acceptance Criteria

### Agent
- [x] `termlink info` shows runtime dir, sessions dir, hub socket path
- [x] Shows hub running/stopped status
- [x] Shows live/stale/total session counts
- [x] Tip to run `clean` when stale sessions exist
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

### 2026-03-08T23:15:38Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-050-cli-info-command--runtime-diagnostics-an.md
- **Context:** Initial task creation

### 2026-03-08T23:17:45Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
