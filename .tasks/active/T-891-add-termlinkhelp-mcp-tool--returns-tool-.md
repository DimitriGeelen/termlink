---
id: T-891
name: "Add termlink_help MCP tool — returns tool catalog for AI agent self-discovery"
description: >
  Add termlink_help MCP tool — returns tool catalog for AI agent self-discovery

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-05T07:35:38Z
last_update: 2026-04-05T07:35:38Z
date_finished: null
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
