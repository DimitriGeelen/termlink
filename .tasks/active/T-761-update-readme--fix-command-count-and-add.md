---
id: T-761
name: "Update README — fix command count and add missing commands to table"
description: >
  Update README — fix command count and add missing commands to table

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-29T20:10:10Z
last_update: 2026-03-29T20:10:10Z
date_finished: null
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
