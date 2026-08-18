---
id: T-848
name: "Add termlink_dispatch_status MCP tool — query dispatch manifest status"
description: >
  Add termlink_dispatch_status MCP tool — query dispatch manifest status

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-04-04T14:55:09Z
last_update: '2026-08-18T18:59:22Z'
date_finished: 2026-04-04T15:11:36Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:11Z'
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
      effort: 1
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=1 
      (no-signal)
    rubric_sha: missing
---

# T-848: Add termlink_dispatch_status MCP tool — query dispatch manifest status

## Context

Read the dispatch manifest at `.termlink/dispatch-manifest.json` and return status summary (pending/merged/conflict/deferred/expired counts, pending branch details). Inlines minimal deserialization types to avoid cross-crate dependency.

## Acceptance Criteria

### Agent
- [x] DUPLICATE — termlink_dispatch_status already exists (added in previous session). No work needed.

## Verification

# No code changes — tool already existed
true

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

### 2026-04-04T14:55:09Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-848-add-termlinkdispatchstatus-mcp-tool--que.md
- **Context:** Initial task creation

### 2026-04-04T15:11:36Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
