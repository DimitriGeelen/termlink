---
id: T-076
name: "Address recurring audit warnings — learnings, practices, historical debt"
description: >
  Address recurring audit warnings — learnings, practices, historical debt

status: started-work
workflow_type: refactor
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-10T13:25:18Z
last_update: 2026-03-10T13:25:18Z
date_finished: null
---

# T-076: Address recurring audit warnings — learnings, practices, historical debt

## Context

8 recurring warnings appeared in both Mar 9 and Mar 10 audits. Categorized as: historical debt (5 unchecked rubber-stamp ACs, 7 lifecycle anomalies, 1 missing research artifact), process gaps (bugfix-learning coverage 0%, focus empty during commits), and tooling gaps (C-003 checkpoint, cron audit).

## Acceptance Criteria

### Agent
- [x] Bugfix learnings registered for T-064 and T-065 (L-002, L-003, L-004)
- [x] Historical rubber-stamp ACs checked for T-027, T-031, T-038, T-046, T-052
- [x] Episodic summaries generated for T-064, T-065, T-075
- [ ] Audit warning count reduced from 15 to fewer than 10

## Verification

# Learnings exist
grep -q 'L-002' .context/project/learnings.yaml
grep -q 'L-003' .context/project/learnings.yaml
# Episodics exist
test -f .context/episodic/T-064.yaml
test -f .context/episodic/T-065.yaml
test -f .context/episodic/T-075.yaml
# No unchecked rubber-stamp ACs in the 5 historical tasks
! grep -l '\- \[ \] \[RUBBER-STAMP\]' .tasks/completed/T-027-*.md .tasks/completed/T-031-*.md .tasks/completed/T-038-*.md .tasks/completed/T-046-*.md .tasks/completed/T-052-*.md 2>/dev/null

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

### 2026-03-10T13:25:18Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-076-address-recurring-audit-warnings--learni.md
- **Context:** Initial task creation
