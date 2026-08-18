---
id: T-775
name: "Add CLI integration tests for uncovered error paths"
description: >
  Add CLI integration tests for uncovered error paths

status: work-completed
workflow_type: test
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-29T23:36:49Z
last_update: '2026-08-18T18:59:21Z'
date_finished: 2026-03-29T23:39:18Z
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
  - ts: '2026-08-18T18:59:21Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 1
      effort: 7
    rationale: blast_radius=0 (no-signal); tier=1 (no-signal); effort=7 
      (no-signal)
    rubric_sha: missing
---

# T-775: Add CLI integration tests for uncovered error paths

## Context

CLI has 75 integration tests but several commands lack error path coverage: agent, push, resize, list --no-header, status --short with session.

## Acceptance Criteria

### Agent
- [x] Add test for `agent ask` with no target (error path)
- [x] Add test for `push` with nonexistent source file (error path)
- [x] Add test for `pty resize` with no target (error path)
- [x] Add test for `list --no-header` output format
- [x] Add test for `status --short` with a live session
- [x] Add test for `list --wait --wait-timeout 1` timeout behavior
- [x] All tests pass: `cargo test -p termlink --test cli_integration` (81 total, 0 failures)

## Verification

cargo test -p termlink --test cli_integration -- cli_agent_ask cli_push_nonexistent cli_pty_resize cli_list_no_header cli_status_short cli_list_wait --quiet

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

### 2026-03-29T23:36:49Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-775-add-cli-integration-tests-for-uncovered-.md
- **Context:** Initial task creation

### 2026-03-29T23:39:18Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
