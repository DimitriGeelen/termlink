---
id: T-158
name: "Session persistence across claude restart"
description: >
  Keep TermLink PTY session alive when claude exits and restarts (claude-fw
  auto-restart). Session, scrollback, and hub registration persist across restarts.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: [remote-access, claude-fw, persistence]
components: []
related_tasks: [T-155, T-156, T-157]
created: 2026-03-17T09:45:47Z
last_update: 2026-03-20T05:58:16Z
date_finished: 2026-03-17T11:35:40Z
---

# T-158: Session persistence across claude restart

## Context

T-156 created `tl-claude.sh` using `termlink spawn -- claude` which kills the
session when claude exits. For claude-fw auto-restart, the session must survive.
Solution: use `--shell` mode (persistent PTY) + inject claude command.

## Acceptance Criteria

### Agent
- [x] `tl-claude.sh start` subcommand for persistent mode
- [x] In persistent mode: spawns shell session, then injects claude command
- [x] `tl-claude.sh restart` re-injects claude into existing session
- [x] `tl-claude.sh status` shows session state
- [x] Session survives claude exit (shell stays alive) — validated in spike
- [x] Scrollback preserved across restarts — validated: both sessions visible in output

### Human
- [x] [REVIEW] Verify session persists across claude exit
  **Steps:**
  1. Run `scripts/tl-claude.sh start --name test-persist`
  2. In Claude, type `/exit` to quit
  3. Run `termlink list` — verify session still exists
  4. Run `scripts/tl-claude.sh restart --name test-persist`
  5. Verify Claude starts again in the same session
  6. Run `termlink pty output test-persist --strip-ansi --lines 20` — verify scrollback has both sessions
  **Expected:** Session persists, scrollback includes both claude invocations
  **If not:** Note which step fails

## Verification

test -x scripts/tl-claude.sh
scripts/tl-claude.sh --help | grep -q "persistent"
scripts/tl-claude.sh --help | grep -q "restart"

## Decisions

### 2026-03-17 — Persistent session approach
- **Chose:** Shell session + inject (no Rust changes needed)
- **Why:** `termlink spawn --shell` keeps PTY alive after command exits; inject new command for restart
- **Rejected:** New `--keep-alive` flag in Rust (unnecessary complexity — shell mode already does this)

## Updates

### 2026-03-17T09:45:47Z — task-created [task-create-agent]
- **Action:** Created task

### 2026-03-17T11:15:29Z — status-update
- **Change:** captured → started-work

### 2026-03-17T11:35:40Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
