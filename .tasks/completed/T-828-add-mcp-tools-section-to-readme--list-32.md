---
id: T-828
name: "Add MCP Tools section to README — list 32 tools for AI agent discoverability"
description: >
  Add MCP Tools section to README — list 32 tools for AI agent discoverability

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-04-03T20:50:31Z
last_update: '2026-08-18T18:59:21Z'
date_finished: 2026-04-03T20:53:26Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:11Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 3
      D4: 3
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=3 
      (body:component-discoverability); D4=3 (body:portability-abstraction); 
      F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:21Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 4
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=4 
      (no-signal)
    rubric_sha: missing
---

# T-828: Add MCP Tools section to README — list 32 tools for AI agent discoverability

## Context

README has no MCP tools listing. Add a section that shows all 32 tools grouped by category, with vendor setup instructions.

## Acceptance Criteria

### Agent
- [x] README has "MCP Server (AI Agent Integration)" section listing all 32 tools
- [x] Tools grouped by category (core, pty, events, metadata, orchestration, self-healing, diagnostics)
- [x] Vendor setup instructions included
- [x] Section placed between CLI Commands and Common Workflows

## Verification

grep -q '32 tools' README.md
grep -q 'collect' README.md
grep -q 'termlink vendor' README.md

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

### 2026-04-03T20:50:31Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-828-add-mcp-tools-section-to-readme--list-32.md
- **Context:** Initial task creation

### 2026-04-03T20:53:26Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
