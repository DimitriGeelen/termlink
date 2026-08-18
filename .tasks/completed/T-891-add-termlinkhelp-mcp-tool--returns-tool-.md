---
id: T-891
name: "Add termlink_help MCP tool — returns tool catalog for AI agent self-discovery"
description: >
  Add termlink_help MCP tool — returns tool catalog for AI agent self-discovery

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-04-05T07:35:38Z
last_update: '2026-08-18T18:59:22Z'
date_finished: 2026-04-05T07:38:42Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:13Z'
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
      blast_radius: 1
      tier: 2
      effort: 4
    rationale: blast_radius=1 (no-signal); tier=2 (no-signal); effort=4 
      (no-signal)
    rubric_sha: missing
---

# T-891: Add termlink_help MCP tool — returns tool catalog for AI agent self-discovery

## Context

AI agents using TermLink MCP have 47 tools. A help tool lets them query available tools by category without reading all schemas.

## Acceptance Criteria

### Agent
- [x] `termlink_help` MCP tool exists, takes optional `category` filter
- [x] Returns categorized tool list with names and short descriptions
- [x] Categories: session, execution, events, kv, files, hub, batch, dispatch, tokens, diagnostics
- [x] `cargo build` succeeds

## Verification

cargo build

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

### 2026-04-05T07:35:38Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-891-add-termlinkhelp-mcp-tool--returns-tool-.md
- **Context:** Initial task creation

### 2026-04-05T07:38:42Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
