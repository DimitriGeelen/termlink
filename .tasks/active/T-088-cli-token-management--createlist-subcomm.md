---
id: T-088
name: "CLI token management — create/list subcommands + client auth"
description: >
  Add termlink token create/list CLI subcommands, client.authenticate() method. From T-079 inception.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-10T23:27:04Z
last_update: 2026-03-11T07:47:17Z
date_finished: null
---

# T-088: CLI token management — create/list subcommands + client auth

## Context

CLI layer for capability tokens. Adds `termlink token create/inspect` and `--token-secret` flag on register. Design: `docs/reports/T-079-capability-tokens.md`

## Acceptance Criteria

### Agent
- [x] `Client::authenticate()` method sends auth.token and returns scope
- [x] `termlink register --token-secret` generates and stores secret in registration
- [x] `termlink token create <target> --scope <scope> --ttl <ttl>` creates signed tokens
- [x] `termlink token inspect <token>` decodes payload without validation
- [x] Workspace builds cleanly, 211 tests pass

## Verification

/Users/dimidev32/.cargo/bin/cargo build -p termlink
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

### 2026-03-10T23:27:04Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-088-cli-token-management--createlist-subcomm.md
- **Context:** Initial task creation

### 2026-03-11T07:47:17Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
