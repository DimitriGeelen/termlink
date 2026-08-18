---
id: T-253
name: "Bypass invalidation signals — config-change-aware cache busting"
description: >
  Bypass registry has no mechanism for external invalidation. Config file changes
  (Cargo.toml, workspace changes) can make bypass entries stale. Add invalidate(pattern)
  method or structured keys with config hash. See docs/reports/T-247-scenarios-code-review.md
  Scenario 2.

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: [T-247, T-238, orchestration, bypass]
components: []
related_tasks: [T-247, T-238, T-233]
created: 2026-03-23T16:54:29Z
last_update: '2026-08-18T18:59:12Z'
date_finished: 2026-03-23T21:34:44Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:50Z'
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
      effort: 6
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=6 
      (no-signal)
    rubric_sha: missing
---

# T-253: Bypass invalidation signals — config-change-aware cache busting

## Context

Bypass entries can become stale after config changes (e.g., workspace restructuring, capability changes). The bypass registry in `crates/termlink-hub/src/bypass.rs` currently has no invalidation mechanism. See `docs/reports/T-247-scenarios-code-review.md` Scenario 2. The `orchestrator.route` handler in `crates/termlink-hub/src/router.rs` needs a new RPC endpoint for external invalidation.

## Acceptance Criteria

### Agent
- [x] `BypassRegistry::invalidate(pattern)` method removes entries matching a case-insensitive substring pattern
- [x] `BypassRegistry::invalidate_all()` clears entire registry
- [x] `orchestrator.bypass_invalidate` RPC endpoint exposed through hub router (with locked_update)
- [x] Test: `invalidate` removes matching entries and preserves non-matching entries
- [x] Test: `invalidate_all` clears everything
- [x] All hub tests pass (`cargo test --package termlink-hub` — 69 tests)

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

### 2026-03-23T16:54:29Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-253-bypass-invalidation-signals--config-chan.md
- **Context:** Initial task creation

### 2026-03-23T21:31:43Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-23T21:34:44Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
