---
id: T-900
name: "Fix agent_ask timeout return to use structured JSON instead of plain text"
description: >
  Fix agent_ask timeout return to use structured JSON instead of plain text

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-04-05T08:51:25Z
last_update: '2026-08-18T18:59:23Z'
date_finished: 2026-04-05T08:55:52Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:13Z'
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
  - ts: '2026-08-18T18:59:23Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 3
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=3 
      (no-signal)
    rubric_sha: missing
---

# T-900: Fix agent_ask timeout return to use structured JSON instead of plain text

## Context

Last plain-text return from T-894/T-895/T-896 standardization sweep. The agent_ask timeout path returns a `format!()` string.

## Acceptance Criteria

### Agent
- [x] agent_ask timeout uses json_err() instead of format!()
- [x] dispatch_status hand-crafted JSON strings replaced with json_err()/to_string_pretty()
- [x] All tests pass (881), zero clippy warnings

## Verification

cargo test --workspace
cargo clippy --workspace --all-targets

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

### 2026-04-05T08:51:25Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-900-fix-agentask-timeout-return-to-use-struc.md
- **Context:** Initial task creation

### 2026-04-05T08:55:52Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
