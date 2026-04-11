---
id: T-903
name: "Extend orchestrator.route with task-type routing"
description: >
  Extend orchestrator.route with task-type routing

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-hub/src/router.rs]
related_tasks: []
created: 2026-04-08T05:57:27Z
last_update: 2026-04-11T14:32:45Z
date_finished: 2026-04-11T14:32:45Z
---

# T-903: Extend orchestrator.route with task-type routing

## Context

Extends the orchestrator.route RPC with optional task_type parameter so the routing chain
can prefer specialists tagged for specific workflow types (build, test, audit, review).
Follows T-902 (MCP task-gate governance). All existing routing works unchanged when task_type is absent.

## Acceptance Criteria

### Agent
- [x] orchestrator.route accepts optional task_type string parameter
- [x] Route cache keys include task_type when present (method::task_type composite key)
- [x] session.discover prefers sessions with matching task-type:<type> tag (sorted first)
- [x] Bypass registry considers task_type in its key for promotion decisions
- [x] Existing routing unchanged when task_type is absent (backward compatible)
- [x] Tests: task-type routing selects correct specialist over generic one
- [x] Tests: fallback to method-only routing when no task-type match exists
- [x] cargo test passes (hub crate)
- [x] Summary report written to docs/reports/T-903-orchestrator-routing.md

## Verification

# Shell commands that MUST pass before work-completed. One per line.
cd /opt/termlink && cargo test -p termlink-hub 2>&1 | tail -5

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

### 2026-04-08T05:57:27Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-903-extend-orchestratorroute-with-task-type-.md
- **Context:** Initial task creation

### 2026-04-11T14:32:45Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
