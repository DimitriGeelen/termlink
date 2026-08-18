---
id: T-846
name: "Add unit tests for protocol events, session filtering, and file chunk math"
description: >
  Add unit tests for protocol events, session filtering, and file chunk math

status: work-completed
workflow_type: test
owner: agent
horizon:
tags: []
components: [crates/termlink-cli/src/commands/file.rs]
related_tasks: []
created: 2026-04-04T13:27:39Z
last_update: '2026-08-18T18:59:22Z'
date_finished: 2026-04-04T13:41:51Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:11Z'
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
  - ts: '2026-08-18T18:59:22Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 1
      effort: 3
    rationale: blast_radius=1 (no-signal); tier=1 (no-signal); effort=3 
      (no-signal)
    rubric_sha: missing
---

# T-846: Add unit tests for protocol events, session filtering, and file chunk math

## Context

Add unit tests to three untested areas: protocol events.rs (ErrorCode, NegotiationState, serde roundtrips), CLI session.rs (filter_sessions), and file.rs chunk calculation extraction.

## Acceptance Criteria

### Agent
- [x] file.rs: Extract calculate_chunks pure function and add 7 unit tests (exact multiple, remainder, single chunk, empty, zero chunk size default, one byte, large file)
- [x] All tests pass: cargo test --workspace (798 tests)
- [x] Zero clippy warnings: cargo clippy --workspace
- Note: session.rs filter_sessions already has 11 tests; events.rs has 31 tests; control.rs has 11 tests; registration.rs has 17 tests; handler.rs has 55 tests — comprehensive coverage already exists across the codebase

## Verification

cargo test --workspace 2>&1 | tail -3
cargo clippy --workspace -- -D warnings 2>&1 | tail -3

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

### 2026-04-04T13:27:39Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-846-add-unit-tests-for-protocol-events-sessi.md
- **Context:** Initial task creation

### 2026-04-04T13:41:51Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
