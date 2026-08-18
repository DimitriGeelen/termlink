---
id: T-860
name: "Add filtering to termlink_list_sessions MCP tool — optional tag, role, name
  params"
description: >
  Add filtering to termlink_list_sessions MCP tool — optional tag, role, name params

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-04-04T19:34:49Z
last_update: '2026-08-18T18:59:22Z'
date_finished: 2026-04-04T19:40:58Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:12Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 0
      D4: 3
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=0 (no-signal); D4=3 
      (body:portability-abstraction); F-RECALL=0 (no-signal); F-ORCH=0 
      (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:22Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 7
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=7 
      (no-signal)
    rubric_sha: missing
---

# T-860: Add filtering to termlink_list_sessions MCP tool — optional tag, role, name params

## Context

`termlink_list_sessions` returns all sessions unfiltered. The CLI `termlink list` supports `--tag`, `--role`, `--name` filtering. Adding optional filter params to the MCP tool makes it useful for targeted session discovery.

## Acceptance Criteria

### Agent
- [x] `ListSessionsParams` added with optional tag, role, name fields
- [x] Filtering logic applied: tag match, role match, name substring match
- [x] Backward compatible — all params optional, no filters = list all
- [x] Integration test for filtered list (by role and name)
- [x] Unit tests for ListSessionsParams (with filters and empty)
- [x] All tests pass: `cargo test -p termlink-mcp` (119 tests)
- [x] Zero clippy warnings

## Verification

cargo test -p termlink-mcp
cargo clippy -p termlink-mcp -- -D warnings

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

### 2026-04-04T19:34:49Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-860-add-filtering-to-termlinklistsessions-mc.md
- **Context:** Initial task creation

### 2026-04-04T19:40:58Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
