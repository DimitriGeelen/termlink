---
id: T-121
name: "PTY mode detection — pty.mode RPC + mode-change events"
description: >
  Add tcgetattr-based terminal mode detection to PtySession. New pty.mode RPC
  returns canonical/raw/echo state. pty.mode-change event emitted on state transitions.
  From T-010 inception GO.
status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [session, pty, interactive, termios]
components: []
related_tasks: [T-010]
created: 2026-03-12T20:17:46Z
last_update: 2026-03-12T20:17:46Z
date_finished: null
---

# T-121: PTY mode detection — pty.mode RPC + mode-change events

## Context

From T-010 inception (docs/reports/T-010-exploration.md). TermLink has zero tcgetattr
usage — completely blind to terminal mode. Agents injecting into vim/REPL sessions
can't distinguish raw vs canonical mode or detect password prompts (echo off).

## Acceptance Criteria

### Agent
- [ ] P1: `pty.mode` RPC returns `{canonical: bool, echo: bool, raw: bool}` via tcgetattr on PTY master fd
- [ ] P2: `pty.mode-change` event emitted when terminal flags change (polled on inject or periodic)
- [ ] P3: Alternate screen buffer detection — track `\e[?1049h/l` in output stream
- [ ] P4: Password prompt hint — event when ECHO flag drops
- [ ] CLI: `termlink status <session>` includes terminal mode in output
- [ ] Tests: mode detection works for canonical (bash) and raw (cat with stty raw) modes

### Human
- [ ] [REVIEW] Verify pty.mode returns correct state during vim session
  **Steps:** Start PTY session, run vim, call pty.mode RPC
  **Expected:** `raw: true, echo: false`
  **If not:** Report actual values

## Verification

/Users/dimidev32/.cargo/bin/cargo test -p termlink-session --lib 2>&1 | tail -1 | grep -q "ok"

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

### 2026-03-12T20:17:46Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-121-pty-mode-detection--ptymode-rpc--mode-ch.md
- **Context:** Initial task creation
