---
id: T-1100
name: "Install updated termlink binary with T-1099 fix"
description: >
  Install updated termlink binary with T-1099 fix

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-16T23:48:05Z
last_update: 2026-04-16T23:54:56Z
date_finished: 2026-04-16T23:54:56Z
---

# T-1100: Install updated termlink binary with T-1099 fix

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] `termlink version` shows 0.9.79 (a563a117) — newer than 0.9.53
- [x] `termlink remote doctor local-test` shows sessions PASS with 8 sessions (was WARN before)

## Verification

termlink version 2>&1 | grep -q "0.9.79"
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

### 2026-04-16T23:48:05Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1100-install-updated-termlink-binary-with-t-1.md
- **Context:** Initial task creation

### 2026-04-16T23:54:56Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
