---
id: T-214
name: "Renumber colliding task IDs T-205–T-210 to T-215–T-220"
description: >
  Renumber colliding task IDs T-205–T-210 to T-215–T-220 to resolve collision with remote tasks

status: started-work
workflow_type: refactor
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-21T23:26:35Z
last_update: 2026-03-21T23:26:35Z
date_finished: null
---

# T-214: Renumber colliding task IDs T-205–T-210 to T-214–T-219

## Context

After rebasing onto origin/main, tasks T-205–T-210 collide with remote tasks created independently. Our tasks are all completed; remote tasks are active. Renumber ours to T-215–T-220 to resolve. Also register G-007 for framework upstream (task counter not safe for concurrent work).

## Acceptance Criteria

### Agent
- [x] 6 task files renamed and IDs updated (T-205→T-215 through T-210→T-220)
- [x] 6 episodic files renamed and task_ids updated (+ new T-216 episodic created)
- [x] 14 fabric cards `created_by` updated from T-206 to T-216
- [x] Handover S-2026-0321-0742 references updated (T-205→T-215)
- [x] No duplicate task IDs in `.tasks/` (verified: 0 duplicates)

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
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-214-renumber-colliding-task-ids-t-205t-210-t.md
- **Context:** Initial task creation
