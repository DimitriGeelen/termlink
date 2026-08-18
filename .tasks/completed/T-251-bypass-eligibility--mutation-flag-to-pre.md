---
id: T-251
name: "Bypass eligibility — mutation flag to prevent read-write commands from promotion"
description: >
  Purely mechanical promotion based on success count cannot distinguish read-only
  from mutating commands. session.cleanup would be incorrectly promoted to Tier 3.
  Add mutating flag to orchestrator.route params or command denylist. See docs/reports/T-247-scenarios-framework-maintenance.md
  Scenario 3.

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: [T-247, T-238, orchestration, bypass]
components: []
related_tasks: [T-247, T-238, T-233]
created: 2026-03-23T16:54:24Z
last_update: '2026-08-18T18:59:12Z'
date_finished: 2026-03-23T17:13:50Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:49Z'
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
  - ts: '2026-08-18T18:59:12Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 5
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=5 
      (no-signal)
    rubric_sha: missing
---

# T-251: Bypass eligibility — mutation flag to prevent read-write commands from promotion

## Context

Mutating commands like `session.cleanup` would be incorrectly promoted to bypass tier by the purely mechanical success-count promotion logic. See `docs/reports/T-247-scenarios-framework-maintenance.md` Scenario 3. Modified files: `crates/termlink-hub/src/bypass.rs`, `crates/termlink-hub/src/router.rs`.

## Acceptance Criteria

### Agent
- [x] `orchestrator.route` accepts optional `mutating: true` param
- [x] When `mutating=true`, bypass check is skipped and orchestrated runs are not tracked for promotion
- [x] Test: mutating command not tracked in candidates after 6 successful runs
- [x] Test: non-mutating command promotes normally after 5 runs, 6th returns bypassed=true
- [x] All 62 hub tests pass

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

### 2026-03-23T16:54:24Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-251-bypass-eligibility--mutation-flag-to-pre.md
- **Context:** Initial task creation

### 2026-03-23T17:10:10Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-23T17:13:50Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
