---
id: T-766
name: "Add strip_ansi_codes unit tests and needs_write tests in handler.rs"
description: >
  Add strip_ansi_codes unit tests and needs_write tests in handler.rs

status: work-completed
workflow_type: test
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-29T22:59:55Z
last_update: '2026-08-18T18:59:20Z'
date_finished: 2026-03-29T23:01:33Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:09Z'
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
  - ts: '2026-08-18T18:59:20Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 1
      effort: 7
    rationale: blast_radius=0 (no-signal); tier=1 (no-signal); effort=7 
      (no-signal)
    rubric_sha: missing
---

# T-766: Add strip_ansi_codes unit tests and needs_write tests in handler.rs

## Context

`strip_ansi_codes` is a pure utility function in handler.rs that sanitizes terminal output by stripping ANSI escape sequences and carriage returns. It's only tested indirectly through `query_output_strip_ansi`. `needs_write` determines write-lock dispatch and has no direct test.

## Acceptance Criteria

### Agent
- [x] Add test for `strip_ansi_codes` with plain text passthrough
- [x] Add test for `strip_ansi_codes` with CSI sequences (color codes, cursor movement)
- [x] Add test for `strip_ansi_codes` with OSC sequences (title setting)
- [x] Add test for `strip_ansi_codes` with carriage return stripping
- [x] Add test for `strip_ansi_codes` with mixed ANSI and plain text
- [x] Add test for `needs_write` identifying correct mutable methods
- [x] All new tests pass via `cargo test -p termlink-session handler`

## Verification

cargo test -p termlink-session handler

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

### 2026-03-29T22:59:55Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-766-add-stripansicodes-unit-tests-and-needsw.md
- **Context:** Initial task creation

### 2026-03-29T23:01:33Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
