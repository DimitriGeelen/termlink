---
id: T-091
name: "Close G-003 — watcher exit-code handling already implemented"
description: >
  Close G-003 — watcher exit-code handling already implemented

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-11T09:48:33Z
last_update: 2026-03-11T23:44:32Z
date_finished: 2026-03-11T23:44:32Z
---

# T-091: Close G-003 — watcher exit-code handling already implemented

## Context

G-003 reported that specialist-watcher.sh and role-watcher.sh emit task.completed unconditionally after Claude exits. Investigation shows both scripts already have exit-code-aware logic: they capture `CLAUDE_EXIT` and emit `task.failed` on non-zero. The gap trigger check (`grep -c 'task.completed'`) is too coarse — it fires because the string exists in the file, even though it's inside a conditional block.

## Acceptance Criteria

### Agent
- [x] Both watchers have exit-code-aware dispatch (confirmed: `CLAUDE_EXIT` capture + conditional emit)
- [x] G-003 closed in gaps.yaml with resolution details
- [x] Trigger check updated to reflect the actual conditional logic

## Verification

grep -q 'CLAUDE_EXIT' tests/e2e/specialist-watcher.sh
grep -q 'task.failed' tests/e2e/specialist-watcher.sh
grep -q 'CLAUDE_EXIT' tests/e2e/role-watcher.sh
grep -q 'task.failed' tests/e2e/role-watcher.sh
python3 -c "import yaml; yaml.safe_load(open('.context/project/gaps.yaml'))"

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

### 2026-03-11T09:48:33Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-091-close-g-003--watcher-exit-code-handling-.md
- **Context:** Initial task creation

### 2026-03-11T23:44:32Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
