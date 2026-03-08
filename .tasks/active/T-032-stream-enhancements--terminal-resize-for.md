---
id: T-032
name: "Stream enhancements — terminal resize forwarding and scrollback catch-up"
description: >
  Stream enhancements — terminal resize forwarding and scrollback catch-up

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-08T20:34:47Z
last_update: 2026-03-08T20:34:47Z
date_finished: null
---

# T-032: Stream enhancements — terminal resize forwarding and scrollback catch-up

## Context

Make `termlink stream` production-quality: forward terminal resize (SIGWINCH → Resize frame) and fetch initial scrollback on connect so the user sees existing output. Predecessor: T-031.

## Acceptance Criteria

### Agent
- [x] SIGWINCH handler sends Resize frame with current terminal dimensions
- [x] Initial scrollback fetched via control plane on connect and printed before streaming
- [x] Resize payload uses big-endian u16 for cols/rows (matches data_server expectation)
- [x] All existing tests pass (110+)
- [x] Builds without warnings

## Verification

/Users/dimidev32/.cargo/bin/cargo build --workspace 2>&1 | grep -E "^(error|warning:)" | head -5; test $? -le 1
/Users/dimidev32/.cargo/bin/cargo test --workspace 2>&1 | tail -5

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

### 2026-03-08T20:34:47Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-032-stream-enhancements--terminal-resize-for.md
- **Context:** Initial task creation
