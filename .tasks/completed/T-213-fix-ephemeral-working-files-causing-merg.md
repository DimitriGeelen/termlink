---
id: T-213
name: "Fix ephemeral working files causing merge conflicts and audit failures"
description: >
  Fix ephemeral working files causing merge conflicts and audit failures

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-21T23:18:01Z
last_update: 2026-03-21T23:19:24Z
date_finished: 2026-03-21T23:19:24Z
---

# T-213: Fix ephemeral working files causing merge conflicts and audit failures

## Context

RCA investigation found that 6 ephemeral session-state files in `.context/working/` are tracked by git but shouldn't be. They cause: (a) merge conflicts during stash/rebase that corrupt JSON (CTL-018 failure), (b) noisy diffs in housekeeping commits, (c) the push-blocking failure chain discovered in this session.

## Acceptance Criteria

### Agent
- [x] `.context/working/.gitignore` updated with 6 missing ephemeral files
- [x] All 6 files removed from git tracking (`git rm --cached`)
- [x] `git ls-files` shows only `.gitignore` tracked in `.context/working/`
- [x] Files still exist on disk (not deleted, just untracked)

## Verification

# Verify no ephemeral files are tracked (only .gitignore should be tracked)
test "$(git ls-files .context/working/ | grep -v .gitignore | wc -l | tr -d ' ')" = "0"
# Verify .gitignore exists and has the new entries
grep -q 'budget-status' .context/working/.gitignore

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

### 2026-03-21T23:18:01Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-213-fix-ephemeral-working-files-causing-merg.md
- **Context:** Initial task creation

### 2026-03-21T23:19:24Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
