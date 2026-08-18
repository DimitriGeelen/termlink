---
id: T-762
name: "Update ARCHITECTURE.md — add MCP crate to hierarchy"
description: >
  Update ARCHITECTURE.md — add MCP crate to hierarchy

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-29T20:14:06Z
last_update: '2026-08-18T18:59:20Z'
date_finished: 2026-03-29T20:16:30Z
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

# T-762: Update ARCHITECTURE.md — add MCP crate to hierarchy

## Context

ARCHITECTURE.md crate hierarchy diagram doesn't include termlink-mcp, and MCP section is missing.

## Acceptance Criteria

### Agent
- [x] termlink-mcp added to crate hierarchy diagram and dependency graph
- [x] MCP section (section 4) added describing purpose, components, resources, and integration
- [x] All 5 crate sections present: protocol, session, hub, mcp, CLI

## Verification

grep -q "termlink-mcp" docs/ARCHITECTURE.md

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

### 2026-03-29T20:14:06Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-762-update-architecturemd--add-mcp-crate-to-.md
- **Context:** Initial task creation

### 2026-03-29T20:16:30Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
