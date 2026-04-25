---
id: T-1281
name: "Promote L-003 to practice — capture subprocess exit codes explicitly"
description: >
  Promote L-003 to practice — capture subprocess exit codes explicitly

status: started-work
workflow_type: refactor
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-25T21:32:44Z
last_update: 2026-04-25T21:32:44Z
date_finished: null
---

# T-1281: Promote L-003 to practice — capture subprocess exit codes explicitly

## Context

L-003 (watcher scripts must capture subprocess exit codes explicitly — `||` swallows non-zero exits and causes false success reporting). 3 apps, D2 Reliability.

## Acceptance Criteria

### Agent
- [x] PP-010 entry created via `fw promote L-003 --directive D2`
- [x] L-003 application field references PP-010

## Verification

test -n "$(grep 'promoted_from: L-003' .context/project/practices.yaml)"
test -n "$(grep -A6 '^- id: L-003' .context/project/learnings.yaml | grep 'application:' | grep -oE 'PP-[0-9]+')"

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

### 2026-04-25T21:32:44Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1281-promote-l-003-to-practice--capture-subpr.md
- **Context:** Initial task creation
