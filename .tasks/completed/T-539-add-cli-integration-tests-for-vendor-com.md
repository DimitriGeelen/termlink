---
id: T-539
name: "Add CLI integration tests for vendor command"
description: >
  Add CLI integration tests for vendor command

status: work-completed
workflow_type: test
owner: agent
horizon:
tags: []
components: [crates/termlink-cli/tests/cli_integration.rs]
related_tasks: []
created: 2026-03-27T19:31:16Z
last_update: '2026-08-18T18:59:17Z'
date_finished: 2026-03-27T19:33:21Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:01Z'
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
  - ts: '2026-08-18T18:59:17Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 1
      effort: 5
    rationale: blast_radius=1 (no-signal); tier=1 (no-signal); effort=5 
      (no-signal)
    rubric_sha: missing
---

# T-539: Add CLI integration tests for vendor command

## Context

No integration tests exist for `termlink vendor`. Add tests covering: fresh vendor, idempotent update, .gitignore creation, MCP config generation, vendor status, and dry-run mode.

## Acceptance Criteria

### Agent
- [x] Tests for fresh vendor (binary + VERSION + .gitignore + MCP config)
- [x] Tests for idempotent re-vendor (no duplicate .gitignore entries)
- [x] Tests for vendor status output
- [x] Tests for dry-run mode (no files created)
- [x] All tests pass: `cargo test --test cli_integration`

## Verification

cargo test --test cli_integration -- vendor 2>&1

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

### 2026-03-27T19:31:16Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-539-add-cli-integration-tests-for-vendor-com.md
- **Context:** Initial task creation

### 2026-03-27T19:33:21Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
