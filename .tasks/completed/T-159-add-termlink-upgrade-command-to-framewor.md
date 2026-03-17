---
id: T-159
name: "Add termlink upgrade command to framework pickup specs"
description: >
  Document the cargo install/upgrade command in framework pickup specs
  (T-148, T-157) so the framework agent knows how to install or upgrade TermLink.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [remote-access, framework]
components: []
related_tasks: [T-148, T-157]
created: 2026-03-17T20:20:50Z
last_update: 2026-03-17T20:21:49Z
date_finished: 2026-03-17T20:21:49Z
---

# T-159: Add termlink upgrade command to framework pickup specs

## Context

Framework agents need to know how to install/upgrade TermLink. Add the cargo command to both pickup specs.

## Acceptance Criteria

### Agent
- [x] T-148 spec has install/upgrade command section
- [x] T-157 pickup prompt has install/upgrade command
- [x] Commands reference both GitHub and local clone paths

## Verification

grep -q "cargo install" docs/specs/T-148-termlink-framework-integration.md
grep -q "cargo install" docs/specs/T-157-claude-fw-termlink-pickup.md

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

### 2026-03-17T20:20:50Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-159-add-termlink-upgrade-command-to-framewor.md
- **Context:** Initial task creation

### 2026-03-17T20:21:49Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
