---
id: T-065
name: "Fix watcher false-completion — detect Claude crash, emit task.failed"
description: >
  Watcher emits task.completed even when claude -p crashes. Check exit code, emit task.failed on non-zero. Add retry option.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-10T08:44:18Z
last_update: 2026-03-10T12:54:46Z
date_finished: 2026-03-10T12:54:46Z
---

# T-065: Fix watcher false-completion — detect Claude crash, emit task.failed

## Context

Reliability flaw found by reflection fleet watcher-pattern agent. Watcher emits task.completed even when claude -p crashes — orchestrator sees false success. See [docs/reports/reflection-result-watcher.md].

## Acceptance Criteria

### Agent
- [x] specialist-watcher.sh captures Claude exit code and emits task.failed on non-zero
- [x] role-watcher.sh captures Claude exit code and emits task.failed on non-zero
- [x] Both watchers include exit_code in task.failed payload
- [x] Successful runs still emit task.completed (no regression)

## Verification

# specialist-watcher checks exit code
grep -q "CLAUDE_EXIT" tests/e2e/specialist-watcher.sh
# specialist-watcher emits task.failed
grep -q "task.failed" tests/e2e/specialist-watcher.sh
# role-watcher checks exit code
grep -q "CLAUDE_EXIT" tests/e2e/role-watcher.sh
# role-watcher emits task.failed
grep -q "task.failed" tests/e2e/role-watcher.sh
# Both include exit_code in failure payload
grep -q "exit_code" tests/e2e/specialist-watcher.sh
grep -q "exit_code" tests/e2e/role-watcher.sh

## Decisions

## Updates

### 2026-03-10T08:44:18Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-065-fix-watcher-false-completion--detect-cla.md
- **Context:** Initial task creation

### 2026-03-10T12:53:31Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-10T12:54:46Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
