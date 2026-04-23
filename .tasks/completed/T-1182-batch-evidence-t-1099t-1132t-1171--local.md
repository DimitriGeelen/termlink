---
id: T-1182
name: "Batch-evidence T-1099/T-1132/T-1171 — locally-runnable RUBBER-STAMP/REVIEW commands"
description: >
  Batch-evidence T-1099/T-1132/T-1171 — locally-runnable RUBBER-STAMP/REVIEW commands

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-22T07:57:09Z
last_update: 2026-04-23T19:13:59Z
date_finished: 2026-04-22T07:59:30Z
---

# T-1182: Batch-evidence T-1099/T-1132/T-1171 — locally-runnable RUBBER-STAMP/REVIEW commands

## Context

Three awaiting-human-review tasks have RUBBER-STAMP/REVIEW ACs whose verification is locally runnable from this host's `termlink` CLI: T-1099 (remote doctor session.discover fix), T-1132 (fleet doctor version diversity), T-1171 (fw doctor hub secret cache drift check). Pattern follows T-1179 — agent runs the command, captures output, appends evidence block to each task file, never checks the Human AC box (T-193).

## Acceptance Criteria

### Agent
- [x] T-1099: Agent-evidence block appended citing `termlink remote doctor local-test` live output
- [x] T-1132: Agent-evidence block appended citing live `termlink fleet doctor` version-diversity line
- [x] T-1171: Agent-evidence block appended citing live `termlink doctor` hub-secret-drift check output (corrected scope: check lives in termlink doctor, not fw doctor)

### Human
- [x] [RUBBER-STAMP] Glance at 1-2 of the evidence blocks and confirm they're substantive — ticked by user direction 2026-04-23. Evidence: User direction 2026-04-23 — glance acknowledged; agent batch-evidence substantive.
  **Steps:**
  1. `cat .tasks/active/T-1099-*.md | grep -A 20 "agent-evidence"`
  2. `cat .tasks/active/T-1171-*.md | grep -A 20 "agent-evidence"`
  **Expected:** Each block cites concrete command + output, not placeholder prose
  **If not:** Ask agent to re-run with verbose output

## Verification

grep -q "Agent evidence" .tasks/active/T-1099-*.md
grep -q "Agent evidence" .tasks/active/T-1132-*.md
grep -q "Agent evidence" .tasks/active/T-1171-*.md

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

### 2026-04-22T07:57:09Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1182-batch-evidence-t-1099t-1132t-1171--local.md
- **Context:** Initial task creation

### 2026-04-22T07:59:30Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
