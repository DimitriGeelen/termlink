---
id: T-055
name: "Interactive TTY integration tests — attach and stream commands via expectrl"
description: >
  Interactive TTY integration tests — attach and stream commands via expectrl

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-09T08:55:10Z
last_update: 2026-03-09T09:42:20Z
date_finished: 2026-03-09T09:42:20Z
---

# T-055: Interactive TTY integration tests — attach and stream commands via expectrl

## Context

Interactive PTY tests for attach and stream commands using rexpect. Extends T-054 integration test harness.

## Acceptance Criteria

### Agent
- [x] rexpect dev-dependency added for PTY-based process spawning
- [x] spawn_termlink helper with proper env var isolation via spawn_command
- [x] wait_for_data_socket utility for data plane readiness (.sock.data)
- [x] attach_shows_output_and_detaches: connect, send command, see output, Ctrl+] detach
- [x] attach_inject_and_see_output: bidirectional I/O through attach
- [x] stream_shows_output_and_detaches: data plane connect, send, see output, detach
- [x] stream_bidirectional_io: full duplex through data plane frames
- [x] Tests marked #[ignore] (require PTY), pass with --ignored
- [x] All 156 standard tests still pass

## Verification

/Users/dimidev32/.cargo/bin/cargo test -p termlink --test interactive_integration -- --ignored 2>&1 | tail -1
/Users/dimidev32/.cargo/bin/cargo test 2>&1 | grep -E "^test result:" | grep -v "0 passed" | head -6

## Decisions

### 2026-03-09 — rexpect over expectrl
- **Chose:** rexpect 0.5 with spawn_command for env var control
- **Why:** Simpler API (send_line, exp_string, send_control), synchronous fits test pattern, spawn_command allows std::process::Command with env vars
- **Rejected:** expectrl (async overkill for sequential test flows), raw pty crate (too low-level)

## Updates

### 2026-03-09T08:55:10Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-055-interactive-tty-integration-tests--attac.md
- **Context:** Initial task creation

### 2026-03-09T09:42:20Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
