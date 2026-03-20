---
id: T-124
name: "dispatch.sh --isolate flag — worktree isolation for mesh workers"
description: >
  dispatch.sh --isolate flag — worktree isolation for mesh workers

status: started-work
workflow_type: build
owner: human
horizon: now
tags: [agent-mesh, isolation, concurrency]
components: []
related_tasks: [T-123, T-114]
created: 2026-03-12T20:57:05Z
last_update: 2026-03-20T13:12:01Z
date_finished: 2026-03-20T13:12:01Z
---

# T-124: dispatch.sh --isolate flag — worktree isolation for mesh workers

## Context

From T-123 inception (docs/reports/T-123-mesh-concurrent-builds.md). Implementing worktree isolation
in dispatch.sh to enable parallel build task execution without file conflicts.

## Acceptance Criteria

### Agent
- [x] dispatch.sh accepts `--isolate` flag
- [x] When `--isolate` is passed: creates git worktree on a branch named `mesh-{worker-name}`
- [x] Sets `CARGO_TARGET_DIR` to worktree-local target directory
- [x] Passes worktree path as workdir to agent-wrapper.sh
- [x] Cleans up worktree on exit (success or failure) via trap
- [x] Without `--isolate`: existing behavior unchanged (backward compatible)
- [x] Worker commits are on the worktree branch, not main

### Human
- [ ] [REVIEW] Dispatch 2 parallel workers with `--isolate` and verify no file conflicts
  **Steps:** Run `dispatch.sh --isolate "echo hello" --worker-name test-a &` and `dispatch.sh --isolate "echo hello" --worker-name test-b &`
  **Expected:** Both complete, two branches created, `git worktree list` shows cleanup
  **If not:** Report which step failed

## Verification

grep -q '\-\-isolate' agents/mesh/dispatch.sh
grep -q 'worktree' agents/mesh/dispatch.sh
grep -q 'CARGO_TARGET_DIR' agents/mesh/dispatch.sh

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

### 2026-03-12T20:57:05Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-124-dispatchsh---isolate-flag--worktree-isol.md
- **Context:** Initial task creation

### 2026-03-12T21:16:09Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed

### 2026-03-20T13:12:01Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
