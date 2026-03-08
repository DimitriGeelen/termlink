---
id: T-017
name: "CLI commands: register, list, ping, status"
description: >
  CLI commands: register, list, ping, status

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-08T16:44:18Z
last_update: 2026-03-08T16:44:18Z
date_finished: null
---

# T-017: CLI commands: register, list, ping, status

## Context

CLI subcommands for TermLink: `register` (start a session), `list` (show sessions), `ping` (verify session), `status` (query session state). Uses clap for arg parsing, connects to session sockets for ping/status.

## Acceptance Criteria

### Agent
- [ ] `termlink register` starts a session with optional `--name` flag
- [ ] `termlink list` shows all registered sessions with state
- [ ] `termlink ping <target>` connects to session socket, sends termlink.ping
- [ ] `termlink status <target>` connects to session socket, sends query.status
- [ ] `--help` works for all subcommands
- [ ] `cargo test --workspace` passes
- [ ] `cargo build --workspace` produces working binary

## Verification

PATH="$HOME/.cargo/bin:$PATH" cargo test --workspace
PATH="$HOME/.cargo/bin:$PATH" cargo clippy --workspace -- -D warnings
PATH="$HOME/.cargo/bin:$PATH" cargo run -- --help

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

### 2026-03-08T16:44:18Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-017-cli-commands-register-list-ping-status.md
- **Context:** Initial task creation
