---
id: T-820
name: "Final CHANGELOG update — complete event.subscribe migration summary"
description: >
  Final CHANGELOG update — complete event.subscribe migration summary

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-30T20:20:13Z
last_update: 2026-03-30T20:21:20Z
date_finished: 2026-03-30T20:21:20Z
---

# T-820: Final CHANGELOG update — complete event.subscribe migration summary

## Context

Update CHANGELOG to reflect T-817 (file receive), T-818 (hub event.collect), T-819 (remote collect). Also add hub event.collect timeout_ms to Added section.

## Acceptance Criteria

### Agent
- [x] CHANGELOG reflects file receive and hub event.collect upgrades
- [x] hub event.collect timeout_ms parameter documented

## Verification

grep -q "event.collect" CHANGELOG.md

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

### 2026-03-30T20:20:13Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-820-final-changelog-update--complete-eventsu.md
- **Context:** Initial task creation

### 2026-03-30T20:21:20Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
