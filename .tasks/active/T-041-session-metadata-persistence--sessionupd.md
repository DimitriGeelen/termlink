---
id: T-041
name: "Session metadata persistence — session.update writes changes to disk"
description: >
  Session metadata persistence — session.update writes changes to disk

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-08T21:36:58Z
last_update: 2026-03-08T21:36:58Z
date_finished: null
---

# T-041: Session metadata persistence — session.update writes changes to disk

## Context

`session.update` RPC mutates in-memory `Registration` (tags, display_name, roles) but never persists to disk. Other sessions reading the JSON file see stale data. Fix: store `registration_path` in `SessionContext`, persist after mutation.

## Acceptance Criteria

### Agent
- [x] `SessionContext` has optional `registration_path: Option<PathBuf>` field
- [x] `handle_session_update` persists to disk after mutation when path is set
- [x] CLI `cmd_register` sets the registration path on SessionContext
- [x] Tests verify disk persistence after session.update
- [x] All existing tests pass (98 unit + 11 integration)

## Verification

/Users/dimidev32/.cargo/bin/cargo test -p termlink-session 2>&1 | tail -1
/Users/dimidev32/.cargo/bin/cargo test -p termlink-cli 2>&1 | tail -1
/Users/dimidev32/.cargo/bin/cargo build 2>&1 | tail -1

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

### 2026-03-08T21:36:58Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-041-session-metadata-persistence--sessionupd.md
- **Context:** Initial task creation
