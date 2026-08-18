---
id: T-550
name: "Add integration tests for --json flags on ping, clean, tag"
description: >
  Add integration tests for --json flags on ping, clean, tag

status: work-completed
workflow_type: test
owner: agent
horizon:
tags: []
components: [crates/termlink-cli/tests/cli_integration.rs]
related_tasks: []
created: 2026-03-28T09:48:18Z
last_update: '2026-08-18T18:59:17Z'
date_finished: 2026-03-28T09:49:27Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:02Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=0 (no-signal); 
      D4=0 (no-signal); F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:17Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 1
      effort: 4
    rationale: blast_radius=1 (no-signal); tier=1 (no-signal); effort=4 
      (no-signal)
    rubric_sha: missing
---

# T-550: Add integration tests for --json flags on ping, clean, tag

## Context

T-546/T-548/T-549 added --json to ping, clean, tag. Need integration test coverage.

## Acceptance Criteria

### Agent
- [x] Test for `ping --json` validates JSON structure (status, latency_ms, id)
- [x] Test for `clean --json` validates JSON structure (dry_run, count, sessions)
- [x] Test for `tag --json` validates JSON structure after tag update
- [x] All tests pass

## Verification

<!-- Shell commands that MUST pass before work-completed. One per line.
     Lines starting with # are comments. Empty lines ignored.
     The completion gate runs each command — if any exits non-zero, completion is blocked.
     Examples:
       python3 -c "import yaml; yaml.safe_load(open('path/to/file.yaml'))"
       curl -sf http://localhost:3000/page
       grep -q "expected_string" output_file.txt
-->

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

### 2026-03-28T09:48:18Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-550-add-integration-tests-for---json-flags-o.md
- **Context:** Initial task creation

### 2026-03-28T09:49:27Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
