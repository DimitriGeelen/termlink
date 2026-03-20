---
id: T-178
name: "Fix pty inject Enter not submitting in Claude Code TUI"
description: >
  pty inject sends text+Enter as one write. Ink TUI needs Enter (0x0D) as separate write with small delay. Root cause: batched write means ink sees multi-char chunk, not a keypress. Fix: split text write and Enter into two separate pty.write() calls. Also check ICRNL termios flag. See docs/reports/T-163-cross-machine-rca-findings.md for full RCA. Related: Claude Code issue #15553, ink useInput batching.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: [bug, cli, inject, pty]
components: []
related_tasks: [T-137, T-156, T-163, T-177]
created: 2026-03-18T22:19:38Z
last_update: 2026-03-20T05:58:18Z
date_finished: 2026-03-18T22:56:00Z
---

# T-178: Fix pty inject Enter not submitting in Claude Code TUI

## Context

RCA report: `docs/reports/T-163-cross-machine-rca-findings.md` (Bug 2 section).
Root cause: ink TUI treats batched text+Enter as paste, not keypress. Fix: split into two separate pty.write() calls with small delay. Also investigate ICRNL termios flag.

## Acceptance Criteria

### Agent
- [x] `handle_command_inject` writes each KeyEntry separately with delay between non-text entries
- [x] Delay is configurable via optional `inject_delay_ms` param (default 10ms)
- [x] Unit test confirms multi-entry inject produces separate writes
- [x] `cargo test --package termlink-session` passes (18/18)
- [x] `cargo build --release` succeeds

### Human
- [x] [REVIEW] Verify Enter submits in Claude Code TUI via pty inject
  **Steps:**
  1. Start a TermLink session: `termlink register --name test-session --shell`
  2. In the session's shell, run `claude`
  3. From another terminal: `termlink pty inject test-session "hello" --enter`
  **Expected:** "hello" appears in Claude Code input AND Enter triggers submission
  **If not:** Check if text appears but Enter doesn't submit — may need longer delay

## Verification

/Users/dimidev32/.cargo/bin/cargo test --package termlink-session --lib inject 2>&1 | grep -q "test result: ok"
/Users/dimidev32/.cargo/bin/cargo build --release 2>&1 | grep -qv "^error"

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

### 2026-03-18T22:19:38Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-178-fix-pty-inject-enter-not-submitting-in-c.md
- **Context:** Initial task creation

### 2026-03-18T22:53:30Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-18T22:56:00Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
