---
id: T-268
name: "CLI integration tests for emit-to, discover, spawn, register --self"
description: >
  CLI integration tests for emit-to, discover, spawn, register --self

status: work-completed
workflow_type: test
owner: agent
horizon:
tags: [testing, cli]
components: []
related_tasks: []
created: 2026-03-24T21:38:02Z
last_update: '2026-08-18T18:59:15Z'
date_finished: 2026-03-24T21:41:13Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:56Z'
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
  - ts: '2026-08-18T18:59:15Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 1
      effort: 3
    rationale: blast_radius=0 (no-signal); tier=1 (no-signal); effort=3 
      (no-signal)
    rubric_sha: missing
---

# T-268: CLI integration tests for emit-to, discover, spawn, register --self

## Context

Add CLI integration tests for features added in T-256, T-263 that lacked E2E coverage.

## Acceptance Criteria

### Agent
- [x] 3 discover tests: by role, by name, JSON output
- [x] 2 register --self tests: endpoint creation + event support
- [x] All 20 CLI integration tests pass (15 existing + 5 new)

## Verification

/Users/dimidev32/.cargo/bin/cargo test -p termlink --test cli_integration -- --test-threads=1 2>&1 | grep -q "20 passed"

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

### 2026-03-24T21:38:02Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-268-cli-integration-tests-for-emit-to-discov.md
- **Context:** Initial task creation

### 2026-03-24T21:41:13Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
