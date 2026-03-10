---
id: T-075
name: "Remediate TODO backlog — enrich handover, task descriptions, traceability"
description: >
  Fill all unfilled TODO/placeholder sections across active tasks and handover.
  Extract concrete acceptance criteria from T-063 reflection fleet reports into
  task files. Ensure every active task has real context, ACs, and verification.

status: work-completed
workflow_type: refactor
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-10T13:03:06Z
last_update: 2026-03-10T13:18:01Z
date_finished: 2026-03-10T13:18:01Z
---

# T-075: Remediate TODO backlog — enrich handover, task descriptions, traceability

## Context

T-063's reflection fleet generated 10 analysis reports in `docs/reports/reflection-result-*.md`. These reports identified concrete gaps that were registered as tasks T-066 through T-073. However, the task files were created with template placeholders — no real ACs, no verification commands, no enriched context. The handover (LATEST.md) also has unfilled TODO sections. This task remediates all quality gaps to ensure traceability and actionability.

## Acceptance Criteria

### Agent
- [x] LATEST.md handover has all TODO sections filled with real content
- [x] T-069 has concrete acceptance criteria (not placeholder text)
- [x] T-066 through T-073 all have concrete ACs derived from reflection reports
- [x] T-008 through T-011 inception tasks have problem statements and key questions
- [x] All active tasks with `workflow_type: build|refactor|specification` have verification commands
- [x] No active task file contains `[First criterion]` or `[Second criterion]` placeholder text

## Verification

# No placeholder ACs remain in any active task
! grep -rl '\[First criterion\]\|\[Second criterion\]\|\[Criterion 1\]' .tasks/active/ --include='*.md' 2>/dev/null | grep -v T-075
# LATEST.md has no unfilled TODO sections
test $(grep -c '\[TODO' .context/handovers/LATEST.md) -eq 0

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

### 2026-03-10T13:03:06Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-075-remediate-todo-backlog--enrich-handover-.md
- **Context:** Initial task creation

### 2026-03-10T13:18:01Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
