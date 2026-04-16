---
id: T-1088
name: "Re-apply T-1066 review-queue patch — third clobber by auto-upgrade"
description: >
  Re-apply T-1066 review-queue patch — third clobber by auto-upgrade

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-16T19:17:41Z
last_update: 2026-04-16T19:17:41Z
date_finished: null
---

# T-1088: Re-apply T-1066 review-queue patch — third clobber by auto-upgrade

## Context

Third time framework auto-upgrade has clobbered the review-queue patch. Recipe in `docs/patches/T-1066-fw-task-review-queue.md`. Related: T-1066, T-1078, G-008, PL-022.

## Acceptance Criteria

### Agent
- [x] `fw task review-queue --count` returns an integer
- [x] `fw task review-queue` shows tagged partial-complete task list
- [x] `fw task help` output includes review-queue entry

## Verification

fw task review-queue --count | grep -qE '^[0-9]+$'
fw task help 2>&1 | grep -q review-queue

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

### 2026-04-16T19:17:41Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1088-re-apply-t-1066-review-queue-patch--thir.md
- **Context:** Initial task creation
