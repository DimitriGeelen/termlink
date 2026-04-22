---
id: T-1185
name: "Finding: R-033 sovereignty gate blocks agent bulk-flip of human-owned tasks — no cleanup possible"
description: >
  Finding: R-033 sovereignty gate blocks agent bulk-flip of human-owned tasks — no cleanup possible

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-22T09:50:12Z
last_update: 2026-04-22T09:52:39Z
date_finished: 2026-04-22T09:52:39Z
---

# T-1185: Finding — R-033 sovereignty gate blocks agent bulk-flip of human-owned tasks

## Context

Investigated an audit D5 warning ("19 lifecycle anomalies") and found 20 tasks in the frontmatter state `status: started-work` + `owner: human` + all Agent ACs `[x]` + 1 unchecked Human RUBBER-STAMP/REVIEW. Scope: T-212, T-1016, T-1027, T-1028, T-1029, T-1030, T-1031, T-1032, T-1040, T-1041, T-1042, T-1043, T-1044, T-1045, T-1046, T-1051, T-1064, T-1074, T-1122, T-1176.

Initial read: "canonical partial-complete is status=work-completed + owner=human (per CTL-025 pass lines), these 20 haven't been flipped, let me batch-flip." Attempted `fw task update T-1042 --status work-completed`. Framework refused:

```
ERROR: Cannot complete human-owned task
Sovereignty gate (R-033): owner is human.
The human must review and approve via Watchtower:
  http://192.168.10.107:3100/review/T-1042
```

**Correct model:** The partial-complete state for these tasks IS `started-work + owner=human + evidence-in-place`. They are waiting for the human to open Watchtower at `:3100/review/T-XXXX`, check the Human AC, and flip the status. The agent's job is to supply evidence (done in prior batch tasks T-1143–T-1147, T-1179, T-1182, T-1184). Flipping status is sovereignty-reserved.

**Why my read was wrong:** `CTL-025: T-1009 partial-complete with owner:human ✓` shows a task that already reached `status: work-completed` — but that terminal state was reached by the human running the flip after Watchtower approval, not by the agent. Current-state introspection of "good" partial-complete tasks does not tell you how they got there.

**Implication for future sessions:** Do not attempt to batch-flip human-owned tasks. D5's "Nd-active" anomalies on these 20 are structural waiting-on-human, not agent work. The action-for-agents is: ensure evidence is in place (done), then wait. Escalation into the 14-day review-queue warning (D2) is a human-review-bandwidth problem, not an agent-cleanup problem.

## Acceptance Criteria

### Agent
- [x] Attempted bulk-flip on T-1042, observed R-033 sovereignty block (evidence in Updates section below)
- [x] Frontmatter scope of 20 candidates validated by scan: started-work + owner:human + all Agent ACs checked
- [x] No Human AC checkboxes were modified, no `--force` attempted, no source files touched
- [x] Finding captured here so future sessions don't re-run the same premise

## Verification

# This task is a finding, not a fix — verification is self-referential
grep -q "Sovereignty gate (R-033)" .tasks/active/T-1185-*.md
grep -q "do not attempt to batch-flip" .tasks/active/T-1185-*.md || grep -q "Do not attempt to batch-flip" .tasks/active/T-1185-*.md

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

### 2026-04-22T09:50:12Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1185-cleanup-flip-20-stuck-partial-complete-t.md
- **Context:** Initial task creation

### 2026-04-22T09:52:39Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
