---
id: T-027
name: "CLI attach command — interactive session with output streaming and input injection"
description: >
  CLI attach command — interactive session with output streaming and input injection

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-08T19:45:06Z
last_update: 2026-03-08T19:47:23Z
date_finished: 2026-03-08T19:47:23Z
---

# T-027: CLI attach command — interactive session with output streaming and input injection

## Context

Interactive attach mode: `termlink attach <target>` shows live terminal output and forwards keystrokes, like a lightweight tmux attach. Uses polling (query.output) and raw terminal mode with Ctrl+] to detach.

## Acceptance Criteria

### Agent
- [x] `Attach` variant in CLI with `target` and `--poll-ms` args
- [x] Raw terminal mode via libc termios (restored on exit)
- [x] Output polling loop shows new content as it arrives
- [x] Stdin forwarded as command.inject text entries
- [x] Ctrl+] (0x1d) detaches cleanly
- [x] Verifies PTY availability before entering attach mode
- [x] Builds and all 102 tests pass

### Human
- [x] [REVIEW] Attach to a PTY session and verify interactive I/O works
  **Steps:**
  1. Terminal A: `termlink register --name test --shell`
  2. Terminal B: `termlink attach test`
  3. Type `ls` and press Enter in Terminal B
  4. Verify output appears in Terminal B
  5. Press Ctrl+] to detach
  **Expected:** Live bidirectional I/O, clean detach
  **If not:** Note which step failed

## Verification

/Users/dimidev32/.cargo/bin/cargo build 2>&1 | tail -1
/Users/dimidev32/.cargo/bin/cargo test --workspace 2>&1 | tail -1

## Decisions

### 2026-03-08 — Polling vs streaming for output
- **Chose:** Polling with configurable interval (default 100ms)
- **Why:** Works over existing control plane without new protocol; data plane streaming is a future enhancement
- **Rejected:** WebSocket/data plane streaming (not built yet), inotify on scrollback (not cross-platform)

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Updates

### 2026-03-08T19:45:06Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-027-cli-attach-command--interactive-session-.md
- **Context:** Initial task creation

### 2026-03-08T19:47:23Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
