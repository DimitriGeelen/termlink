---
id: T-290
name: "Fix pre-push audit context — hook cannot find tasks dir, blocks git push"
description: >
  Fix pre-push audit context — hook cannot find tasks dir, blocks git push

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-25T22:49:16Z
last_update: 2026-03-25T22:49:16Z
date_finished: null
---

# T-290: Fix pre-push audit context — hook cannot find tasks dir, blocks git push

## Context

Pre-push hook runs audit which fails to find .tasks/ — sources lib/paths.sh which doesn't exist in consumer projects. Blocks git push with false FAILs.

## Acceptance Criteria

### Agent
- [x] `git push origin main` succeeds without `--no-verify`
- [x] Pre-push hook correctly resolves PROJECT_ROOT and finds .tasks/
- [x] `fw audit` still passes

## Verification

PROJECT_ROOT=/Users/dimidev32/001-projects/010-termlink /usr/local/opt/agentic-fw/libexec/bin/fw audit

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

### 2026-03-25T22:49:16Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-290-fix-pre-push-audit-context--hook-cannot-.md
- **Context:** Initial task creation
