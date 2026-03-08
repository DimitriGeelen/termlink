---
id: T-050
name: "CLI info command — runtime diagnostics and system overview"
description: >
  CLI info command — runtime diagnostics and system overview

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-08T23:15:38Z
last_update: 2026-03-08T23:15:38Z
date_finished: null
---

# T-050: CLI info command — runtime diagnostics and system overview

## Context

Quick health check for the TermLink system — shows runtime paths, hub status, session counts.

## Acceptance Criteria

### Agent
- [x] `termlink info` shows runtime dir, sessions dir, hub socket path
- [x] Shows hub running/stopped status
- [x] Shows live/stale/total session counts
- [x] Tip to run `clean` when stale sessions exist
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

### 2026-03-08T23:15:38Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-050-cli-info-command--runtime-diagnostics-an.md
- **Context:** Initial task creation
