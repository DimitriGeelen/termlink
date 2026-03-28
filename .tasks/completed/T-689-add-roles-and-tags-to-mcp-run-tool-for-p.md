---
id: T-689
name: "Add roles and tags to MCP run tool for parity with CLI"
description: >
  Add roles and tags to MCP run tool for parity with CLI

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T23:37:04Z
last_update: 2026-03-28T23:37:04Z
date_finished: 2026-03-29T00:18:00Z
---

# T-689: Add roles and tags to MCP run tool for parity with CLI

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] Not applicable — MCP run uses direct execution (executor::execute), not session registration. Roles/tags need a session to attach to.

## Verification

<!-- Shell commands that MUST pass before work-completed. One per line.
     Lines starting with # are comments. Empty lines ignored.
     The completion gate runs each command — if any exits non-zero, completion is blocked.
     Examples:
       python3 -c "import yaml; yaml.safe_load(open('path/to/file.yaml'))"
       curl -sf http://localhost:3000/page
       grep -q "expected_string" output_file.txt
-->

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

### 2026-03-28T23:37:04Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-689-add-roles-and-tags-to-mcp-run-tool-for-p.md
- **Context:** Initial task creation
