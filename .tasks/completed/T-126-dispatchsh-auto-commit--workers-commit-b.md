---
id: T-126
name: "dispatch.sh auto-commit — workers commit before cleanup"
description: >
  Auto-commit worker changes before worktree cleanup. From T-123 retrospective.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: [agent-mesh, isolation]
components: []
related_tasks: []
created: 2026-03-13T10:05:12Z
last_update: 2026-03-20T05:58:13Z
date_finished: 2026-03-14T11:57:34Z
---

# T-126: dispatch.sh auto-commit — workers commit before cleanup

## Context

From T-123 retrospective: worktree agents completed work but didn't commit, requiring manual commit + rebase. Auto-commit in cleanup ensures work is preserved on the branch.

## Acceptance Criteria

### Agent
- [x] dispatch.sh auto-commits worker changes in worktree before cleanup (when --isolate)
- [x] Commit message includes worker name and task context
- [x] No-op when worker made no changes (no empty commits)
- [x] Worktree branch preserved (not removed) when commits exist, so merge orchestration can pick it up
- [x] Worktree still cleaned up when no changes were made

### Human
- [x] [RUBBER-STAMP] Run `dispatch.sh --isolate --worker-name test-commit "echo hello > /tmp/test.txt"` and verify branch has commit
  **Steps:**
  1. Run dispatch with --isolate and a prompt that modifies a file
  2. Check `git log mesh-test-commit --oneline -1`
  **Expected:** Branch exists with auto-commit
  **If not:** Check dispatch.sh stderr for commit errors

## Verification

# Auto-commit function exists in dispatch.sh
grep -q 'git.*commit' agents/mesh/dispatch.sh
# Cleanup still works for no-change case
grep -q 'worktree remove' agents/mesh/dispatch.sh

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

### 2026-03-13T10:05:12Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-126-dispatchsh-auto-commit--workers-commit-b.md
- **Context:** Initial task creation

### 2026-03-13T10:06:54Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-13T10:07:35Z — status-update [task-update-agent]
- **Change:** status: started-work → captured

### 2026-03-14T11:44:10Z — status-update [task-update-agent]
- **Change:** horizon: later → now

### 2026-03-14T11:44:17Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-14T11:57:34Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
