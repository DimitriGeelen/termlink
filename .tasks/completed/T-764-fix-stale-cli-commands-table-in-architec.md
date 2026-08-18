---
id: T-764
name: "Fix stale CLI commands table in ARCHITECTURE.md"
description: >
  Fix stale CLI commands table in ARCHITECTURE.md

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-29T20:28:06Z
last_update: '2026-08-18T18:59:20Z'
date_finished: 2026-03-29T20:29:06Z
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
      effort: 2
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=2 
      (no-signal)
    rubric_sha: missing
---

# T-764: Fix stale CLI commands table in ARCHITECTURE.md

## Context

ARCHITECTURE.md CLI commands table is stale — missing dispatch, mirror, agent, file, remote, doctor, vendor, mcp, version.

## Acceptance Criteria

### Agent
- [x] CLI commands table updated to match actual 30 commands (12 groups)
- [x] All command groups represented including Agent, Files, Remote, Tools

## Verification

grep -q "doctor" docs/ARCHITECTURE.md

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

### 2026-03-29T20:28:06Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-764-fix-stale-cli-commands-table-in-architec.md
- **Context:** Initial task creation

### 2026-03-29T20:29:06Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
