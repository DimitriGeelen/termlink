---
id: T-202
name: "termlink remote exec — execute commands on remote sessions"
description: >
  termlink remote exec — execute commands on remote sessions

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: [cli, cross-machine, remote]
components: []
related_tasks: []
created: 2026-03-20T23:54:27Z
last_update: 2026-03-23T10:24:13Z
date_finished: 2026-03-21T00:01:13Z
---

# T-202: termlink remote exec — execute commands on remote sessions

## Context

Execute shell commands on remote sessions via hub routing. Forwards `command.execute` with
`target` param. Mirrors local `exec` behavior: prints stdout/stderr, exits with remote exit code.

## Acceptance Criteria

### Agent
- [x] `RemoteAction::Exec` variant with args: hub, session, command, secret-file/secret, scope, timeout, cwd, json
- [x] `cmd_remote_exec()` connects, authenticates, forwards `command.execute` with target routing
- [x] stdout/stderr printed to respective streams
- [x] Process exits with remote command's exit code on non-zero
- [x] `--json` outputs full result (exit_code, stdout, stderr)
- [x] `cargo build --package termlink` compiles clean
- [x] `cargo test --workspace` passes (297 passed, 0 failed)
- [x] Help text: `termlink remote exec --help`

### Human
- [x] [REVIEW] Cross-machine test: execute command on .107 session
  **Steps:**
  1. `termlink remote exec 192.168.10.107:9100 fw-agent "hostname" --secret-file /tmp/termlink-107-secret.txt`
  2. Verify output shows .107's hostname
  **Expected:** Remote hostname printed
  **If not:** Check session has `allowed_commands` permitting the command

## Verification

/Users/dimidev32/.cargo/bin/cargo build --package termlink
grep -q "RemoteAction::Exec" crates/termlink-cli/src/main.rs

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

### 2026-03-20T23:54:27Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-202-termlink-remote-exec--execute-commands-o.md
- **Context:** Initial task creation

### 2026-03-21T00:01:13Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
