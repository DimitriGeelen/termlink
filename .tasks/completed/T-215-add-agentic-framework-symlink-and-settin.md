---
id: T-215
name: "Add .agentic-framework symlink and settings backup to gitignore"
description: >
  Add .agentic-framework symlink and settings backup to gitignore

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-22T17:28:21Z
last_update: 2026-03-22T17:29:02Z
date_finished: 2026-03-22T17:29:02Z
---

# T-215: Add .agentic-framework symlink and settings backup to gitignore

## Context

Machine-specific symlink (.agentic-framework) and Claude Code backup file should not be committed.

## Acceptance Criteria

### Agent
- [x] `.agentic-framework` in .gitignore
- [x] `.claude/settings.json.bak` in .gitignore

## Verification

grep -q ".agentic-framework" .gitignore
grep -q "settings.json.bak" .gitignore

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

### 2026-03-22T17:28:21Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-215-add-agentic-framework-symlink-and-settin.md
- **Context:** Initial task creation

### 2026-03-22T17:29:02Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
