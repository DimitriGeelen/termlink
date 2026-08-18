---
id: T-225
name: "Add unit tests for CLI utility functions"
description: >
  Add unit tests for CLI utility functions

status: work-completed
workflow_type: test
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-21T10:34:47Z
last_update: '2026-08-18T18:59:06Z'
date_finished: 2026-03-21T10:36:52Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:35Z'
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
  - ts: '2026-08-18T18:59:06Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 1
      effort: 6
    rationale: blast_radius=0 (no-signal); tier=1 (no-signal); effort=6 
      (no-signal)
    rubric_sha: missing
---

# T-225: Add unit tests for CLI utility functions

## Context

T-222 extracted `util.rs` with 7 utility functions but only 5 ANSI-stripping tests. Add tests for `truncate`, `parse_signal`, `shell_escape`, `resize_payload`, and `generate_request_id`.

## Acceptance Criteria

### Agent
- [x] Tests for `truncate()` — normal, at-boundary, over-boundary, and empty cases
- [x] Tests for `parse_signal()` — numeric, named, SIG-prefix, case-insensitive, invalid
- [x] Tests for `shell_escape()` — safe strings, whitespace, single quotes, special chars
- [x] Tests for `resize_payload()` — standard, large, and roundtrip encoding
- [x] Tests for `generate_request_id()` — format validation, uniqueness with delay
- [x] All 23 tests pass (18 new + 5 existing ANSI tests)

## Verification

/Users/dimidev32/.cargo/bin/cargo test -p termlink util:: --manifest-path /Users/dimidev32/001-projects/010-termlink/Cargo.toml

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

### 2026-03-21T10:34:47Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-225-add-unit-tests-for-cli-utility-functions.md
- **Context:** Initial task creation

### 2026-03-21T10:36:52Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
