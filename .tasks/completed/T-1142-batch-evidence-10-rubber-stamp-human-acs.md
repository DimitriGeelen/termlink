---
id: T-1142
name: "Batch-evidence 10 RUBBER-STAMP Human ACs (G-008 remediation)"
description: >
  Batch-evidence 10 RUBBER-STAMP Human ACs (G-008 remediation)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [g-008, batch-evidence, human-ac-review]
components: [.tasks/active/T-1008*.md, .tasks/active/T-1009*.md, .tasks/active/T-1010*.md, .tasks/active/T-1011*.md, .tasks/active/T-1021*.md, .tasks/active/T-1034*.md, .tasks/active/T-1035*.md, .tasks/active/T-1038*.md, .tasks/active/T-1040*.md, .tasks/active/T-1102*.md]
related_tasks: []
created: 2026-04-19T16:51:31Z
last_update: 2026-04-19T16:53:29Z
date_finished: 2026-04-19T16:53:29Z
---

# T-1142: Batch-evidence 10 RUBBER-STAMP Human ACs (G-008 remediation)

## Context

G-008 lists 64+ tasks stuck in partial-complete because Human ACs were never
rubber-stamped. This task takes 10 of them that are objectively verifiable
from this workstation and injects agent-gathered evidence directly under the
relevant Human AC so the human's review cost drops to "read the evidence line
and tick." We never tick on their behalf.

## Acceptance Criteria

### Agent
- [x] `/tmp/batch-evidence.py` runs cleanly and reports `updated 10/10 task files`
- [x] Every targeted task file contains a line matching `Agent evidence (auto-batch 2026-04-19, G-008 remediation,`
- [x] Every targeted task's original Human AC boxes are still unchecked (we add evidence, not ticks)

## Verification

test -x /tmp/batch-evidence.py
bash -c 'count=0; for id in T-1008 T-1009 T-1010 T-1011 T-1021 T-1034 T-1035 T-1038 T-1040 T-1102; do grep -q "G-008 remediation" /opt/termlink/.tasks/active/${id}-*.md 2>/dev/null && count=$((count+1)); done; [ "$count" = "10" ]'

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

### 2026-04-19T16:51:31Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1142-batch-evidence-10-rubber-stamp-human-acs.md
- **Context:** Initial task creation

### 2026-04-19T16:53:29Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
