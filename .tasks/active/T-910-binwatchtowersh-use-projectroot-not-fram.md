---
id: T-910
name: "bin/watchtower.sh use PROJECT_ROOT not FRAMEWORK_ROOT for PID/log"
description: >
  Follow-up from T-909. bin/watchtower.sh:16-22 uses $FRAMEWORK_ROOT for the PID file and log paths, causing cross-project collisions when multiple projects share a framework (the symlink case in T-909 proved this: running watchtower.sh stop from one project could kill another project's instance). Fix: use $PROJECT_ROOT (or PROJECT_ROOT env override) for PID/log; fall back to FRAMEWORK_ROOT only as last resort. Add regression test.

status: started-work
workflow_type: build
owner: human
horizon: now
tags: [infrastructure, watchtower, path-resolution]
components: []
related_tasks: []
created: 2026-04-11T12:28:29Z
last_update: 2026-04-12T12:52:10Z
date_finished: null
---

# T-910: bin/watchtower.sh use PROJECT_ROOT not FRAMEWORK_ROOT for PID/log

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] PID_FILE uses `$PROJECT_ROOT/.context/working/watchtower.pid` (not FRAMEWORK_ROOT)
- [x] LOG_FILE uses `$PROJECT_ROOT/.context/working/watchtower.log` (not FRAMEWORK_ROOT)
- [x] PROJECT_ROOT is resolved before PID/LOG paths are set (via paths.sh on line 18)

## Verification

# Shell commands that MUST pass before work-completed. One per line.
grep -q 'PROJECT_ROOT.*watchtower.pid' /opt/termlink/.agentic-framework/bin/watchtower.sh
! grep -q 'FRAMEWORK_ROOT.*watchtower.pid' /opt/termlink/.agentic-framework/bin/watchtower.sh

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

### 2026-04-11T12:28:29Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-910-binwatchtowersh-use-projectroot-not-fram.md
- **Context:** Initial task creation

### 2026-04-12T11:21:31Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
