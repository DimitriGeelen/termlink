---
id: T-248
name: "Fix bypass registry write race — atomic file operations"
description: >
  BypassRegistry load/modify/save is not atomic. Concurrent orchestrator.route calls
  can silently lose promotions. Fix with write-to-temp + atomic rename + file locking.
  See docs/reports/T-247-scenarios-adversarial.md Scenario 1, T-247-scenarios-code-review.md
  Scenario 3, T-247-scenarios-research.md Scenario 3.

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: [T-247, T-238, orchestration, bypass, bug]
components: []
related_tasks: [T-247, T-238, T-233]
created: 2026-03-23T16:54:17Z
last_update: '2026-08-18T18:59:11Z'
date_finished: 2026-03-23T17:02:23Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:47Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 1
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 2
      F-ORCH: 0
    rationale: D1=1 (body:fix-without-learning); D2=0 (no-signal); D3=0 
      (no-signal); D4=0 (no-signal); F-RECALL=2 (body:lightly-promoted); 
      F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:11Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 6
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=6 
      (no-signal)
    rubric_sha: missing
---

# T-248: Fix bypass registry write race — atomic file operations

## Context

High-severity bug found by 3/5 scenario agents in T-247 orchestration scenario research. The bypass registry's `save_to()` uses truncate+write which races under concurrent `orchestrator.route` calls, silently losing promotion data. See `docs/reports/T-247-scenarios-adversarial.md` Scenario 1 for reproduction details. Modified files: `crates/termlink-hub/src/bypass.rs`.

## Acceptance Criteria

### Agent
- [x] `save_to()` uses write-to-temp + atomic rename (not truncate+write)
- [x] `load_from()` handles corrupt/partial JSON gracefully (returns default, logs warning)
- [x] File locking (advisory flock) via `locked_update()` around load+modify+save cycle
- [x] Router uses `locked_update()` for all bypass registry mutations
- [x] Concurrent write test: 10 parallel `record_orchestrated_run` calls, verify no data loss
- [x] All 59 hub tests pass (0 warnings)

## Verification

/Users/dimidev32/.cargo/bin/cargo test --package termlink-hub

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

### 2026-03-23T16:54:17Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-248-fix-bypass-registry-write-race--atomic-f.md
- **Context:** Initial task creation

### 2026-03-23T16:59:20Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-23T17:02:23Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
