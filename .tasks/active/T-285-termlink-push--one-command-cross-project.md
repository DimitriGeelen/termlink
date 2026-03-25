---
id: T-285
name: "termlink push — one-command cross-project file delivery with PTY notification"
description: >
  termlink push — one-command cross-project file delivery with PTY notification

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-25T20:02:11Z
last_update: 2026-03-25T20:02:11Z
date_finished: null
---

# T-285: termlink push — one-command cross-project file delivery with PTY notification

## Context

T-283 investigation revealed cross-project notification is broken: 3-4 fragile commands, escaping issues, `send-file` doesn't auto-materialize, no delivery confirmation. This command replaces that with one atomic operation.

## Acceptance Criteria

### Agent
- [ ] `termlink push --help` shows usage
- [ ] `termlink push <hub-or-profile> <session> <file>` delivers file to target's inbox via `remote exec`
- [ ] After delivery, injects one-line PTY notification: `[TERMLINK] Received: <filename> — cat <path>`
- [ ] Reports delivery confirmation to sender (file path, size, target)
- [ ] `--message` flag allows inline text push without a file
- [ ] `--json` flag for structured output
- [ ] Uses profile-based auth (resolves hub profiles like other remote commands)
- [ ] Inbox path is `/tmp/termlink-inbox/` on target (created if missing)
- [ ] All existing tests pass (`cargo test --workspace`)
- [ ] 0 compiler warnings

### Human
- [ ] [REVIEW] Push a file from .112 to fw-agent on .107, verify file arrives and agent sees notification
  **Steps:**
  1. `echo "test push" > /tmp/push-test.md`
  2. `termlink push mint fw-agent /tmp/push-test.md`
  3. `termlink remote exec mint fw-agent "cat /tmp/termlink-inbox/push-test.md"`
  **Expected:** File content matches, push reports success
  **If not:** Check `termlink remote list mint` for connectivity

## Verification

/Users/dimidev32/.cargo/bin/cargo test --workspace

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

### 2026-03-25T20:02:11Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-285-termlink-push--one-command-cross-project.md
- **Context:** Initial task creation
