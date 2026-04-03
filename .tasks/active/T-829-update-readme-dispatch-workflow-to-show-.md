---
id: T-829
name: "Update README dispatch workflow to show termlink dispatch command"
description: >
  Update README dispatch workflow to show termlink dispatch command

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-03T20:56:09Z
last_update: 2026-04-03T20:56:09Z
date_finished: null
---

# T-829: Update README dispatch workflow to show termlink dispatch command

## Context

README "Dispatch parallel workers" section shows old manual spawn+wait pattern. Update to show `termlink dispatch` (atomic spawn+tag+collect) and `--isolate` for worktree isolation.

## Acceptance Criteria

### Agent
- [x] README dispatch section shows `termlink dispatch` as the primary approach
- [x] Shows `--isolate` option for git worktree isolation
- [x] Old manual pattern kept as collapsible alternative

## Verification

grep -q 'termlink dispatch' README.md
grep -q 'isolate' README.md

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

### 2026-04-03T20:56:09Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-829-update-readme-dispatch-workflow-to-show-.md
- **Context:** Initial task creation
