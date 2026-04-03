---
id: T-828
name: "Add MCP Tools section to README — list 32 tools for AI agent discoverability"
description: >
  Add MCP Tools section to README — list 32 tools for AI agent discoverability

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-03T20:50:31Z
last_update: 2026-04-03T20:50:31Z
date_finished: null
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
grep -q 'termlink_collect' README.md
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
