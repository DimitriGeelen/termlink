---
id: T-127
name: "Merge orchestration script — rebase + sequential merge N worktree branches"
description: >
  Script to rebase and merge N worktree branches onto main after parallel dispatch. From T-123 retrospective.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: [agent-mesh, isolation]
components: []
related_tasks: []
created: 2026-03-13T10:05:22Z
last_update: 2026-03-20T13:12:02Z
date_finished: 2026-03-20T13:12:02Z
---

# T-127: Merge orchestration script — rebase + sequential merge N worktree branches

## Context

From T-123 retrospective: after parallel dispatch with --isolate, N worktree branches need rebasing and sequential merging onto main. This was done manually last session — automate it.

## Acceptance Criteria

### Agent
- [x] `agents/mesh/merge-branches.sh` exists and is executable
- [x] Accepts branch names as arguments (e.g., `merge-branches.sh mesh-worker-1 mesh-worker-2`)
- [x] Rebases each branch onto main sequentially (not parallel — conflicts need resolution)
- [x] Runs test suite after each merge to catch breakage early
- [x] Stops on first conflict or test failure with clear error message
- [x] Reports summary: merged count, remaining branches, test results
- [x] Cleans up merged branches after successful merge

### Human
- [x] [RUBBER-STAMP] Run after a real parallel dispatch and verify branches merge cleanly
  **Steps:**
  1. Dispatch 2+ workers with --isolate
  2. Run `merge-branches.sh mesh-worker-1 mesh-worker-2`
  **Expected:** Branches merged, tests pass, branches cleaned up
  **If not:** Check stderr for conflict details

## Verification

# Script exists and is executable
test -x agents/mesh/merge-branches.sh
# Script has rebase logic
grep -q 'rebase' agents/mesh/merge-branches.sh
# Script runs tests
grep -q 'test --workspace' agents/mesh/merge-branches.sh
bash -n agents/mesh/merge-branches.sh

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

### 2026-03-13T10:05:22Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-127-merge-orchestration-script--rebase--sequ.md
- **Context:** Initial task creation

### 2026-03-14T11:57:51Z — status-update [task-update-agent]
- **Change:** horizon: later → now

### 2026-03-14T11:57:52Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-14T12:04:21Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed

### 2026-03-20T13:12:02Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
