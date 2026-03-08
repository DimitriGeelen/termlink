---
id: T-049
name: "CLI topics command — list event topics from sessions"
description: >
  CLI topics command — list event topics from sessions

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-08T23:05:21Z
last_update: 2026-03-08T23:13:53Z
date_finished: 2026-03-08T23:13:53Z
---

# T-049: CLI topics command — list event topics from sessions

## Context

Debug tool: see what event topics exist across sessions, useful for understanding event flows.

## Acceptance Criteria

### Agent
- [x] `termlink topics` queries all sessions for their event topics
- [x] `termlink topics <target>` queries a specific session
- [x] Output grouped by session name with topic list
- [x] Summary line shows total topics and sessions
- [x] All tests pass

## Verification

/Users/dimidev32/.cargo/bin/cargo build -p termlink 2>&1 | tail -1
/Users/dimidev32/.cargo/bin/cargo test 2>&1 | grep -E "^test result:" | grep -v "0 passed" | head -4

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

### 2026-03-08T23:05:21Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-049-cli-topics-command--list-event-topics-fr.md
- **Context:** Initial task creation

### 2026-03-08T23:13:53Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
