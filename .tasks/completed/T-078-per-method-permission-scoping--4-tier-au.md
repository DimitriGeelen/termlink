---
id: T-078
name: "Per-method permission scoping — 4-tier auth in RPC dispatch"
description: >
  Map 17 RPC methods to 4 permission tiers (observe/interact/control/execute). Add
  scope checking to handler dispatch. Use AUTH_DENIED error code. Depends on T-077.

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-10T20:44:17Z
last_update: '2026-08-18T18:58:41Z'
date_finished: 2026-03-10T20:58:37Z
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
      effort: 6
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=6 
      (no-signal)
    rubric_sha: missing
---

# T-078: Per-method permission scoping — 4-tier auth in RPC dispatch

## Context

Phase 2 of security model (T-008 GO). Adds 4-tier permission scoping (observe/interact/control/execute) to RPC dispatch. See [docs/reports/T-008-security-model-inception.md].

## Acceptance Criteria

### Agent
- [x] `PermissionScope` enum with 4 tiers in auth.rs (observe, interact, control, execute)
- [x] `method_scope()` function mapping all RPC methods to their required scope
- [x] Scope hierarchy: execute > control > interact > observe (higher grants lower)
- [x] `PeerCredentials` extended with granted scopes (default: all scopes for same-UID)
- [x] Unit tests for method-to-scope mapping
- [x] All existing tests pass (cargo test --workspace)

## Verification

/Users/dimidev32/.cargo/bin/cargo build --workspace 2>&1 | tail -1
/Users/dimidev32/.cargo/bin/cargo test --workspace 2>&1 | grep "test result:" | grep -v "0 passed"

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

### 2026-03-10T20:44:17Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-078-per-method-permission-scoping--4-tier-au.md
- **Context:** Initial task creation

### 2026-03-10T20:56:25Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-10T20:58:37Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
