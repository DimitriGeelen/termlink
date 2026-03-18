---
id: T-170
name: "Fix PTY test failures under parallel execution"
description: >
  Fix PTY test failures under parallel execution

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-18T12:12:43Z
last_update: 2026-03-18T16:03:31Z
date_finished: 2026-03-18T16:03:31Z
---

# T-170: Fix PTY test failures under parallel execution

## Context

PTY tests `terminal_mode_returns_valid_flags` and `write_and_read_roundtrip` fail under parallel
execution due to PTY device exhaustion on macOS. Root cause: no Drop impl on PtySession (zombie
processes, leaked PTY devices) and 5-second read_loop timeouts holding devices too long.

## Acceptance Criteria

### Agent
- [x] PtySession implements Drop — kills child process and reaps (waitpid) to release PTY device
- [x] Test timeouts reduced from 5s to 2s where tests use read_loop/wait
- [x] All PTY tests pass in parallel mode (`cargo test -p termlink-session`)
- [x] No zombie shell processes left after test run

## Verification

bash -c 'out=$(/Users/dimidev32/.cargo/bin/cargo test --package termlink-session 2>&1); echo "$out" | grep -q "0 failed"'
grep -q "impl Drop for PtySession" crates/termlink-session/src/pty.rs

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

### 2026-03-18T12:12:43Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-170-fix-pty-test-failures-under-parallel-exe.md
- **Context:** Initial task creation

### 2026-03-18T16:03:31Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
