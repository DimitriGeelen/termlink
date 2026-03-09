---
id: T-031
name: "CLI stream subcommand — real-time data plane attach via binary frames"
description: >
  CLI stream subcommand — real-time data plane attach via binary frames

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-08T20:31:15Z
last_update: 2026-03-08T20:33:11Z
date_finished: 2026-03-08T20:33:11Z
---

# T-031: CLI stream subcommand — real-time data plane attach via binary frames

## Context

Real-time data plane attach via binary frames. Unlike the polling-based `attach` command, `stream` connects directly to the data socket for zero-latency bidirectional I/O. Predecessor: T-029 (data plane), T-030 (data plane wiring).

## Acceptance Criteria

### Agent
- [x] `Stream` subcommand added to CLI with target argument
- [x] Connects to data socket (`{control_socket}.data`) resolved from session registration
- [x] Raw terminal mode with restore on exit
- [x] Streams Output frames to stdout in real-time
- [x] Forwards stdin as Input frames
- [x] Ctrl+] (0x1d) detaches cleanly
- [x] Sends Close frame on detach
- [x] All existing tests pass (110+)
- [x] Builds without warnings

### Human
- [ ] [RUBBER-STAMP] Interactive streaming works end-to-end
  **Steps:**
  1. Terminal A: `termlink register --name test-stream --shell`
  2. Terminal B: `termlink stream test-stream`
  3. Type commands in Terminal B, verify output appears
  4. Press Ctrl+] to detach
  **Expected:** Real-time I/O, clean detach, terminal restored
  **If not:** Note which step failed and any error messages

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

### 2026-03-08T20:31:15Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-031-cli-stream-subcommand--real-time-data-pl.md
- **Context:** Initial task creation

### 2026-03-08T20:33:11Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
