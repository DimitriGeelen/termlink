---
id: T-794
name: "Add dispatch status CLI and audit check for orphaned branches"
description: >
  Phase 5: dispatch status subcommand + audit section for orphaned branches

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: [T-789, T-793]
created: 2026-03-30T13:35:20Z
last_update: 2026-03-30T14:13:09Z
date_finished: 2026-03-30T14:13:00Z
---

# T-794: Add dispatch status CLI and audit check for orphaned branches

## Context

Phase 5 of T-789. The CLI status command was delivered in T-793 (`termlink dispatch-status`). This task updates ARCHITECTURE.md and CHANGELOG.md with the new worktree isolation feature, new commands, and updated test counts.

## Acceptance Criteria

### Agent
- [x] ARCHITECTURE.md updated with new test count
- [x] CHANGELOG.md updated with worktree isolation features (--isolate, --auto-merge, --workdir, dispatch-status)
- [x] `dispatch-status` command documented
- [x] New manifest module documented

## Verification

grep -q "dispatch-status" CHANGELOG.md
grep -q "isolate" CHANGELOG.md

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

### 2026-03-30T13:35:20Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-794-add-dispatch-status-cli-and-audit-check-.md
- **Context:** Initial task creation

### 2026-03-30T14:11:19Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-30T14:13:00Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
