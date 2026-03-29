---
id: T-762
name: "Update ARCHITECTURE.md — add MCP crate to hierarchy"
description: >
  Update ARCHITECTURE.md — add MCP crate to hierarchy

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-29T20:14:06Z
last_update: 2026-03-29T20:14:06Z
date_finished: null
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
