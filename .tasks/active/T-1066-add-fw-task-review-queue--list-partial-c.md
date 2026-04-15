---
id: T-1066
name: "Add fw task review-queue — list partial-complete tasks awaiting human signature (G-008 mitigation)"
description: >
  Add fw task review-queue — list partial-complete tasks awaiting human signature (G-008 mitigation)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-15T18:48:40Z
last_update: 2026-04-15T18:48:40Z
date_finished: null
---

# T-1066: Add fw task review-queue — list partial-complete tasks awaiting human signature (G-008 mitigation)

## Context

G-008 mitigation: 64 tasks stuck in partial-complete with no surface to prompt review. Add `fw task review-queue` that lists them, sorted by date_finished ASC (oldest first), tagged RUBBER-STAMP / REVIEW / MIXED / UNTAGGED. Supports `--count` for handover digest and `--rubber-stamp-only` to focus on trivial ticks.

## Acceptance Criteria

### Agent
- [x] `fw task review-queue` lists partial-complete (status=work-completed) tasks with unchecked Human ACs
- [x] Output is sorted by date_finished ASC (oldest first)
- [x] Each row shows: `[TAG]` (RUBBER-STAMP/REVIEW/MIXED/UNTAGGED), T-ID, owner, unchecked/total, name, age in days
- [x] `--count` flag prints just the integer count (for scripting/handover digest)
- [x] `--rubber-stamp-only` filters to only RUBBER-STAMP-tagged tasks
- [x] Command appears in `fw task help` output
- [x] Running on current repo produces a non-empty, sorted list (61 tasks: 39 rubber-stamp, 22 review)

## Verification

./.agentic-framework/bin/fw task review-queue --count | grep -qE '^[0-9]+$'
./.agentic-framework/bin/fw task review-queue --rubber-stamp-only 2>&1 | grep -q 'Review Queue\|No tasks awaiting'
./.agentic-framework/bin/fw task help 2>&1 | grep -q 'review-queue'

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

### 2026-04-15T18:48:40Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1066-add-fw-task-review-queue--list-partial-c.md
- **Context:** Initial task creation
