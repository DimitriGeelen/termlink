---
id: T-992
name: "Session housekeeping — episodics, fabric drift, stale tasks"
description: >
  Session housekeeping — episodics, fabric drift, stale tasks

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-13T06:11:18Z
last_update: 2026-04-13T06:11:18Z
date_finished: null
---

# T-992: Session housekeeping — episodics, fabric drift, stale tasks

## Context

Address audit warnings: 2 missing episodics (T-990, T-991), fabric drift (3 files), stale/bypassed tasks (T-854, T-931, T-243, T-260).

## Acceptance Criteria

### Agent
- [x] Episodic summaries generated for T-990 and T-991
- [x] Fabric drift resolved (fw fabric drift shows 0 unregistered)
- [x] T-854 and T-931 placeholder ACs backfilled and checked
- [x] T-243 and T-260 reviewed (both already horizon: later; T-260 now has verification section)

## Verification

test -f .context/episodic/T-990.yaml
test -f .context/episodic/T-991.yaml

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

### 2026-04-13T06:11:18Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-992-session-housekeeping--episodics-fabric-d.md
- **Context:** Initial task creation
