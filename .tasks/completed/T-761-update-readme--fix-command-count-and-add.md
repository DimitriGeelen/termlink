---
id: T-761
name: "Update README — fix command count and add missing commands to table"
description: >
  Update README — fix command count and add missing commands to table

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-29T20:10:10Z
last_update: '2026-08-18T18:59:20Z'
date_finished: 2026-03-29T20:11:22Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:09Z'
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
  - ts: '2026-08-18T18:59:20Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 3
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=3 
      (no-signal)
    rubric_sha: missing
---

# T-761: Update README — fix command count and add missing commands to table

## Context

README says "26 commands" but actual count is 30. Command table is also missing several commands added since the original README.

## Acceptance Criteria

### Agent
- [x] Command count updated from 26 to 30 in architecture diagram
- [x] Missing commands added to CLI Commands table (mirror, dispatch, signal, agent, file, remote, doctor, vendor, mcp, version)
- [x] MCP crate added to architecture table

## Verification

grep -q "30 commands" README.md

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

### 2026-03-29T20:10:10Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-761-update-readme--fix-command-count-and-add.md
- **Context:** Initial task creation

### 2026-03-29T20:11:22Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
