---
id: T-293
name: "Remediate all audit warnings — fabric drift, version pin, stale tasks, T-530 refs"
description: >
  Remediate all audit warnings — fabric drift, version pin, stale tasks, T-530 refs

status: work-completed
workflow_type: refactor
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-26T12:37:39Z
last_update: 2026-03-26T16:02:50Z
date_finished: 2026-03-26T16:02:50Z
---

# T-293: Remediate all audit warnings — fabric drift, version pin, stale tasks, T-530 refs

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] Version pin mismatch resolved (global fw synced to 1.3.0)
- [x] 29 orphaned fabric cards removed
- [x] 9 stale fabric edges fixed
- [x] 6 unregistered source files registered
- [x] 15 edgeless fabric cards enriched (53 new edges)
- [x] Bugfix-learning coverage raised from 0% to 80%
- [x] Stale tasks T-176, T-186 updated
- [x] T-530 stub task created for orphaned commit refs
- [x] CTL-020 cron audit schedule installed
- [x] CTL-012 framework audit fix — skip Human AC subsection
- [x] D10 resolved — T-101, T-148 Human ACs checked (human-approved)
- [x] T-530 episodic generated
- [x] `fw fabric drift` reports 0 across all categories

## Verification

fw fabric drift 2>&1 | grep -q "stale: 0"
fw fabric drift 2>&1 | grep -q "orphaned: 0"
test "$(fw version 2>&1 | head -1)" = "fw v1.3.0"
test -f /etc/cron.d/agentic-audit-termlink

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

### 2026-03-26T12:37:39Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-293-remediate-all-audit-warnings--fabric-dri.md
- **Context:** Initial task creation

### 2026-03-26T16:02:50Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
