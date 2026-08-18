---
id: T-991
name: "fw upgrade vendor step errors instead of skipping when source=target (self-hosted
  framework)"
description: >
  fw upgrade vendor step errors instead of skipping when source=target (self-hosted
  framework)

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-04-13T06:00:33Z
last_update: '2026-08-18T18:59:24Z'
date_finished: 2026-04-13T06:02:12Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:16Z'
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
  - ts: '2026-08-18T18:59:24Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 3
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=3 
      (no-signal)
    rubric_sha: missing
---

# T-991: fw upgrade vendor step errors instead of skipping when source=target (self-hosted framework)

## Context

`fw upgrade` step 4b calls `do_vendor --source "$FRAMEWORK_ROOT"` but when the project IS the framework host (`.agentic-framework` is both source and target), `do_vendor` errors with "Source and target resolve to the same directory" instead of skipping gracefully. This blocks `fw upgrade` on self-hosted setups like termlink on .107.

## Acceptance Criteria

### Agent
- [x] `fw upgrade` succeeds (exit 0) on self-hosted framework projects where FRAMEWORK_ROOT = target/.agentic-framework
- [x] Step 4b prints a clear OK message instead of ERROR when source = target
- [x] `fw upgrade` still works correctly on consumer projects (do_vendor still called when source != target)

## Verification

# fw upgrade must complete without error on this project
.agentic-framework/bin/fw upgrade 2>&1 | grep -q "ERROR" && exit 1 || exit 0

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

### 2026-04-13T06:00:33Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-991-fw-upgrade-vendor-step-errors-instead-of.md
- **Context:** Initial task creation

### 2026-04-13T06:02:12Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
