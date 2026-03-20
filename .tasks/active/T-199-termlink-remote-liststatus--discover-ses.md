---
id: T-199
name: "termlink remote list/status — discover sessions on remote hubs"
description: >
  termlink remote list/status — discover sessions on remote hubs

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: [cli, cross-machine, remote]
components: []
related_tasks: []
created: 2026-03-20T23:25:30Z
last_update: 2026-03-20T23:33:17Z
date_finished: 2026-03-20T23:33:17Z
---

# T-199: termlink remote list/status — discover sessions on remote hubs

## Context

Natural extension of the `remote` subcommand family (inject, send-file). Users need to discover
what sessions exist on a remote hub before they can interact with them. Reuses the existing
TOFU+auth connection pattern and the hub's `session.discover` RPC method.

## Acceptance Criteria

### Agent
- [x] `RemoteAction::List` variant added with args: hub, secret-file/secret, scope, json, name/tags/roles filters
- [x] `RemoteAction::Status` variant added with args: hub, session, secret-file/secret, scope, json
- [x] `cmd_remote_list()` connects via TOFU TLS, authenticates, calls `session.discover`, displays results
- [x] `cmd_remote_status()` connects, authenticates, forwards `query.status` with `target` param
- [x] Table output matches local `list` format (ID, NAME, STATE, PID, TAGS)
- [x] `--json` flag outputs structured JSON
- [x] `cargo build --package termlink` compiles clean
- [x] `cargo test --package termlink` passes (297 passed, 0 failed)
- [x] Help text: `termlink remote list --help` and `termlink remote status --help`
- [x] Refactored: extracted `connect_remote_hub()` helper, removed ~120 lines of duplicated TOFU+auth boilerplate from inject and send-file

### Human
- [ ] [REVIEW] Cross-machine test: list sessions on .107
  **Steps:**
  1. Ensure hub running on .107 with `--tcp` and a registered session
  2. `termlink remote list 192.168.10.107:9100 --secret-file /tmp/termlink-107-secret.txt`
  3. Verify sessions appear in table format
  **Expected:** Sessions on .107 listed with correct names and states
  **If not:** Check hub logs, verify auth, try with `--json`

## Verification

/Users/dimidev32/.cargo/bin/cargo build --package termlink
grep -q "RemoteAction::List" crates/termlink-cli/src/main.rs
grep -q "cmd_remote_list" crates/termlink-cli/src/main.rs
grep -q "cmd_remote_status" crates/termlink-cli/src/main.rs

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

### 2026-03-20T23:25:30Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-199-termlink-remote-liststatus--discover-ses.md
- **Context:** Initial task creation

### 2026-03-20T23:33:17Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
