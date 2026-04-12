---
id: T-949
name: "Pickup: fw inception decide fails on captured tasks — should auto-transition through started-work (from termlink)"
description: >
  Auto-created from pickup envelope. Source: termlink, task T-940. Type: bug-report.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [pickup, bug-report, framework]
components: []
related_tasks: []
created: 2026-04-12T08:12:01Z
last_update: 2026-04-12T13:01:27Z
date_finished: 2026-04-12T13:01:27Z
---

# T-949: Pickup: fw inception decide fails on captured tasks — should auto-transition through started-work (from termlink)

## Context

Bug: `fw inception decide T-XXX go` fails when task is in `captured` status because `update-task.sh` enforces valid transitions (`captured → started-work → work-completed`) and there's no direct `captured → work-completed` path. Fix: auto-transition through `started-work` before completing.

## Acceptance Criteria

### Agent
- [x] Root cause identified: `enums.sh` defines `captured → started-work` only, no direct `captured → work-completed`
- [x] Fix applied: `inception.sh` auto-transitions `captured → started-work` before `work-completed`
- [x] `cargo build --workspace` not affected (framework-only change)

## Verification

# Shell commands that MUST pass before work-completed. One per line.
grep -q 'Auto-transition.*captured.*started-work' /opt/termlink/.agentic-framework/lib/inception.sh

## Decisions

### 2026-04-12 — Fix approach
- **Chose:** Auto-transition in inception.sh (check current status, call update-task twice if captured)
- **Why:** Minimal change, preserves state machine integrity in enums.sh
- **Rejected:** Adding captured→work-completed to enums.sh (would bypass started-work tracking)

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-04-12T13:00:07Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now

### 2026-04-12T13:01:27Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
