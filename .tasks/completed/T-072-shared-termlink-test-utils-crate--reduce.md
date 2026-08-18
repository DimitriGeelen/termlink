---
id: T-072
name: "Shared termlink-test-utils crate — reduce test boilerplate"
description: >
  Create workspace crate for shared test helpers: unique dirs, process guards, socket
  polling, session fixtures.

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-10T08:44:48Z
last_update: '2026-08-18T18:58:41Z'
date_finished: 2026-03-11T10:05:25Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:40Z'
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

# T-072: Shared termlink-test-utils crate — reduce test boilerplate

## Context

Test infrastructure gap found by reflection fleet architecture and test-coverage agents. No shared test-utils crate, boilerplate repeated across crates. See [docs/reports/reflection-result-arch.md] and [docs/reports/reflection-result-testcov.md].

## Acceptance Criteria

### Agent
- [x] `termlink-test-utils` crate exists in workspace with `[dev-dependencies]` usage from other crates
- [x] Unique temp dir helper: creates `/tmp/tl-test-*` dirs with auto-cleanup on drop
- [x] Process guard helper: spawns a process and kills it on drop (RAII cleanup)
- [x] Socket polling helper: waits for a Unix socket to become available with timeout
- [x] Session fixture helper: registers a session with default config, returns handle for testing
- [x] At least 2 existing test files refactored to use the shared helpers (proving the abstraction works)
- [x] All existing tests pass after refactoring

## Verification

/Users/dimidev32/.cargo/bin/cargo test -p termlink-test-utils 2>&1 | tail -5
grep -q 'termlink-test-utils' Cargo.toml

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

### 2026-03-10T08:44:48Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-072-shared-termlink-test-utils-crate--reduce.md
- **Context:** Initial task creation

### 2026-03-11T09:56:40Z — status-update [task-update-agent]
- **Change:** horizon: later → now

### 2026-03-11T09:56:44Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-11T10:05:25Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
