---
id: T-819
name: "Upgrade remote collect to use timeout_ms for push-based delivery"
description: >
  Upgrade remote collect to use timeout_ms for push-based delivery

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-30T20:18:11Z
last_update: 2026-03-30T20:18:11Z
date_finished: null
---

# T-819: Upgrade remote collect to use timeout_ms for push-based delivery

## Context

Remote `cmd_remote_collect` uses sleep loop with `event.collect`. Pass `timeout_ms` for push-based delivery.

## Acceptance Criteria

### Agent
- [x] Remote collect passes `timeout_ms` to `event.collect`
- [x] Sleep removed from remote collect loop
- [x] `cargo check -p termlink` passes

## Verification

cargo check -p termlink 2>&1 | grep -q "Finished"

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

### 2026-03-30T20:18:11Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-819-upgrade-remote-collect-to-use-timeoutms-.md
- **Context:** Initial task creation
