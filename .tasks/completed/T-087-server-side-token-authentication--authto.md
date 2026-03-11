---
id: T-087
name: "Server-side token authentication — auth.token RPC method"
description: >
  Add auth.token RPC method, connection scope upgrade from token, default scope logic (Execute if no secret, Observe if secret). From T-079 inception.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-10T23:27:02Z
last_update: 2026-03-11T07:46:47Z
date_finished: 2026-03-11T07:46:47Z
---

# T-087: Server-side token authentication — auth.token RPC method

## Context

Server-side integration of capability tokens (T-086). Adds `auth.token` RPC method, connection scope upgrade, and conditional default scope. Design: `docs/reports/T-079-capability-tokens.md`

## Acceptance Criteria

### Agent
- [x] `auth.token` RPC method constant added to protocol crate
- [x] `handle_connection` supports mutable scope — upgradeable via auth.token
- [x] Sessions with `token_secret` default to Observe scope (not Execute)
- [x] Sessions without `token_secret` preserve legacy Execute scope
- [x] Permission denied errors use AUTH_DENIED (-32010) error code
- [x] 5 integration tests: scope upgrade, observe-only, wrong secret, no secret, accept loop

## Verification

/Users/dimidev32/.cargo/bin/cargo test -p termlink-session -- server
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

### 2026-03-10T23:27:02Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-087-server-side-token-authentication--authto.md
- **Context:** Initial task creation

### 2026-03-11T07:41:02Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-11T07:46:47Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
