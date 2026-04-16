---
id: T-1079
name: "Propagate 1-minute pickup cron to termlink-connected agents"
description: >
  Propagate 1-minute pickup cron to termlink-connected agents

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-16T05:34:48Z
last_update: 2026-04-16T05:36:21Z
date_finished: 2026-04-16T05:36:21Z
---

# T-1079: Propagate 1-minute pickup cron to termlink-connected agents

## Context

T-1073 set pickup cron to 1 minute locally. Propagate this config as a pickup envelope to all reachable termlink-connected agents so they also drain their inboxes every minute instead of the default cadence.

## Acceptance Criteria

### Agent
- [x] Pickup envelope created with 1-minute cron instructions
- [x] Envelope sent to reachable termlink peers via file send (2 local sessions; ring20-dashboard auth-fail, ring20-management down)

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.

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

### 2026-04-16T05:34:48Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1079-propagate-1-minute-pickup-cron-to-termli.md
- **Context:** Initial task creation

### 2026-04-16T05:36:21Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
