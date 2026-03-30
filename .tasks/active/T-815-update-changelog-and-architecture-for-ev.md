---
id: T-815
name: "Update CHANGELOG and ARCHITECTURE for event.subscribe migration"
description: >
  Update CHANGELOG and ARCHITECTURE for event.subscribe migration

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-30T19:50:46Z
last_update: 2026-03-30T19:50:46Z
date_finished: null
---

# T-815: Update CHANGELOG and ARCHITECTURE for event.subscribe migration

## Context

Update CHANGELOG and ARCHITECTURE docs to reflect T-811/T-812/T-813/T-814 event.subscribe migration and correct test count (684 not 688).

## Acceptance Criteria

### Agent
- [x] CHANGELOG reflects CLI and MCP tools event.subscribe migration
- [x] CHANGELOG test count corrected to 684
- [x] ARCHITECTURE test counts current (session 250→251)

## Verification

grep -q "684" CHANGELOG.md

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

### 2026-03-30T19:50:46Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-815-update-changelog-and-architecture-for-ev.md
- **Context:** Initial task creation
