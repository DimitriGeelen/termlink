---
id: T-234
name: "termlink mirror — read-only terminal mirroring (single session, local+remote)"
description: >
  termlink mirror — read-only terminal mirroring (single session, local+remote)

status: started-work
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-23T09:09:49Z
last_update: 2026-03-23T10:24:23Z
date_finished: null
---

# T-234: termlink mirror — read-only terminal mirroring (single session, local+remote)

## Context

Build task from T-232 inception (GO). Add `termlink mirror <session>` — read-only data plane streaming that lets you observe agent terminal output without interfering. Design: docs/reports/T-232-terminal-shadow-sessions.md

## Acceptance Criteria

### Agent
- [x] Data plane supports read-only "mirror" connection mode (client-enforced: no Input/Signal/Resize sent)
- [x] CLI command `termlink mirror <session>` streams terminal output read-only
- [x] Mirror works via local Unix socket data plane
- [x] Observe permission scope grants data plane read-only access (data plane is open, same as stream)
- [x] Multiple mirror clients can connect simultaneously to same session
- [x] Initial scrollback catch-up on connect (shows recent output before streaming)
- [x] Ctrl+C cleanly disconnects mirror without affecting mirrored session
- [x] Unit tests for mirror mode (multi-subscriber verified)
- [x] `termlink mirror --help` shows usage

### Human
- [x] [REVIEW] Test mirror on a live agent session
  **Steps:**
  1. Start a TermLink session: `termlink register --name test-agent -- bash`
  2. In another terminal: `termlink mirror test-agent`
  3. Type commands in the agent terminal, observe they appear in the mirror
  4. Try typing in the mirror terminal — keystrokes should NOT reach the agent
  5. Ctrl+C the mirror — agent session should continue unaffected
  **Expected:** Real-time output streaming, read-only, clean disconnect
  **If not:** Check data plane socket exists, check permission scope

## Verification

# Mirror command exists in CLI
grep -q 'cmd_mirror\|"mirror"' crates/termlink-cli/src/commands/pty.rs
# Mirror mode in data server
grep -q 'mirror\|Mirror\|ReadOnly\|read_only' crates/termlink-session/src/data_server.rs
# Tests exist
grep -rq 'mirror' crates/termlink-session/src/data_server.rs crates/termlink-hub/src/router.rs

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

### 2026-03-23T09:09:49Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-234-termlink-mirror--read-only-terminal-mirr.md
- **Context:** Initial task creation

### 2026-03-23T09:15:51Z — status-update [task-update-agent]
- **Change:** owner: agent → human
