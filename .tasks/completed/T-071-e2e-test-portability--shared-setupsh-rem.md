---
id: T-071
name: "E2E test portability — shared setup.sh, remove hardcoded paths"
description: >
  Extract shared orchestrator registration and health check into setup.sh. Replace
  hardcoded paths with env vars for portability.

status: work-completed
workflow_type: refactor
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-10T08:44:46Z
last_update: '2026-08-18T18:58:41Z'
date_finished: 2026-03-14T11:16:31Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:40Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 0
      D4: 1
      F-RECALL: 2
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=0 (no-signal); D4=1 
      (body:hard-coded-removed); F-RECALL=2 (body:lightly-promoted); F-ORCH=0 
      (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:41Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 3
      effort: 6
    rationale: blast_radius=0 (no-signal); tier=3 (no-signal); effort=6 
      (no-signal)
    rubric_sha: missing
---

# T-071: E2E test portability — shared setup.sh, remove hardcoded paths

## Context

Portability issue found by reflection fleet e2e-suite agent. Hardcoded paths and duplicated setup patterns across e2e levels. See [docs/reports/reflection-result-e2e.md].

## Acceptance Criteria

### Agent
- [x] Shared `setup.sh` exists with common orchestrator registration, health check, and binary resolution
- [x] All 6 e2e level scripts source `setup.sh` instead of duplicating setup logic
- [x] No hardcoded absolute paths (e.g., `/Users/dimidev32/...`) in any e2e test script
- [x] Binary paths resolved via `$PATH`, `which`, or `TERMLINK_BIN`/`CLAUDE_BIN` env vars
- [x] All e2e tests pass after portability changes

## Verification

# No hardcoded user paths in e2e scripts
! grep -r '/Users/dimidev32' tests/e2e/*.sh 2>/dev/null
# setup.sh exists and is sourced
test -f tests/e2e/setup.sh
grep -q 'source.*setup.sh\|\. .*setup.sh' tests/e2e/level1-echo.sh

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

### 2026-03-10T08:44:46Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-071-e2e-test-portability--shared-setupsh-rem.md
- **Context:** Initial task creation

### 2026-03-12T18:56:40Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-12T19:02:54Z — status-update [task-update-agent]
- **Change:** horizon: later → now

### 2026-03-14T11:16:31Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
