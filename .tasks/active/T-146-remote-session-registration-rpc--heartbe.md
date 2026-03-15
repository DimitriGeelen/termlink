---
id: T-146
name: "Remote session registration RPC + heartbeat"
description: >
  Remote session registration RPC with heartbeat and TTL

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [tcp, hub]
components: []
related_tasks: []
created: 2026-03-15T22:06:26Z
last_update: 2026-03-15T22:12:06Z
date_finished: null
---

# T-146: Remote session registration RPC + heartbeat

## Context

Hub-mediated registration for remote TCP sessions. Remote sessions register
via RPC, heartbeat to stay alive, auto-expire after TTL. See T-144 inception.

## Acceptance Criteria

### Agent
- [x] In-memory remote session store in hub (thread-safe, TTL-based)
- [x] `session.register_remote` RPC method stores remote session entry
- [x] `session.heartbeat` RPC method refreshes TTL
- [x] `session.deregister_remote` RPC method removes entry
- [x] Background reaper task expires stale entries (default TTL: 5 min, reap every 30s)
- [x] `session.discover` returns both local FS and remote sessions
- [x] All existing tests pass (262 total)
- [x] 5 new tests for remote store (register, heartbeat, deregister, expiry, JSON format)

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

### 2026-03-15T22:06:26Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-146-remote-session-registration-rpc--heartbe.md
- **Context:** Initial task creation

### 2026-03-15T22:12:06Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
