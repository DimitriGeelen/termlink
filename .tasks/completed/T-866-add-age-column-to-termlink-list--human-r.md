---
id: T-866
name: "Add AGE column to termlink list — human-readable session uptime"
description: >
  Add AGE column to termlink list — human-readable session uptime

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/session.rs]
related_tasks: []
created: 2026-04-04T21:26:46Z
last_update: 2026-04-04T21:31:41Z
date_finished: 2026-04-04T21:31:41Z
---

# T-866: Add AGE column to termlink list — human-readable session uptime

## Context

`termlink list` table doesn't show session age — with 60 sessions it's hard to tell which are fresh (minutes) vs stale (days/weeks). Add an AGE column with human-readable relative timestamps.

## Acceptance Criteria

### Agent
- [x] `format_age` utility function converts Unix timestamp to "Xs/Xm/Xh/Xd" relative time string
- [x] `termlink list` table includes AGE column between STATE and PID
- [x] Unit tests for `format_age` edge cases (seconds, minutes, hours, days, invalid, future)
- [x] Zero clippy warnings

## Verification

grep -q 'format_age' crates/termlink-cli/src/commands/session.rs
grep -q 'AGE' crates/termlink-cli/src/commands/session.rs

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

### 2026-04-04T21:26:46Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-866-add-age-column-to-termlink-list--human-r.md
- **Context:** Initial task creation

### 2026-04-04T21:31:41Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
