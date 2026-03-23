---
id: T-229
name: "Renumber colliding task IDs T-205–T-210 to T-222–T-227"
description: >
  Renumber colliding task IDs T-205–T-210 to T-222–T-227 to resolve collision with remote tasks

status: work-completed
workflow_type: refactor
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-21T23:26:35Z
last_update: 2026-03-23T07:47:02Z
date_finished: 2026-03-23T07:47:02Z
---

# T-229: Renumber colliding task IDs T-205–T-210 to T-222–T-227

## Context

After rebasing onto origin/main, tasks T-205–T-210 collide with remote tasks created independently. Our tasks are all completed; remote tasks are active. Renumber ours to T-222–T-227 to resolve. Also register G-007 for framework upstream (task counter not safe for concurrent work).

## Acceptance Criteria

### Agent
- [x] 6 task files renamed and IDs updated (T-215→T-222 through T-220→T-227)
- [x] 6 episodic files renamed and task_ids updated
- [x] 14 fabric cards `created_by` updated from T-216 to T-223
- [x] Handover S-2026-0321-0742 references updated (T-215→T-222)
- [x] No duplicate task IDs in `.tasks/` (verified: 0 duplicates)
- [x] 6 stale active/ copies of remote-completed tasks removed (T-208, T-216–T-220)

## Verification

# No duplicate IDs across active + completed
test "$(cat .tasks/active/*.md .tasks/completed/*.md 2>/dev/null | grep '^id:' | sort | uniq -d | wc -l | tr -d ' ')" = "0"

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

### 2026-03-21T23:26:35Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-229-renumber-colliding-task-ids-t-205t-210-t.md
- **Context:** Initial task creation

### 2026-03-23T07:47:02Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
