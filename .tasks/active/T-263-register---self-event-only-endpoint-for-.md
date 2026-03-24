---
id: T-263
name: "register --self: event-only endpoint for existing processes"
description: >
  Library API + CLI flag: any process becomes a TermLink endpoint (events, KV, discovery) without PTY. From T-262 GO decision.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [cli, session, orchestration]
components: []
related_tasks: [T-262, T-233]
created: 2026-03-24T10:47:34Z
last_update: 2026-03-24T10:47:34Z
date_finished: null
---

# T-263: register --self: event-only endpoint for existing processes

## Context

From T-262 GO decision. See `docs/reports/T-262-attach-self-inception.md`. Any process becomes a TermLink endpoint (events, KV, discovery) without PTY ownership. Composition over ownership — TermLink becomes a capability OF the process.

## Acceptance Criteria

### Agent
- [x] `--self` flag on `register` CLI command (mutually exclusive with `--shell`)
- [x] Self-registered session runs RPC server in current process (Unix socket, hub discovery)
- [x] Capabilities: events (emit/poll), KV (get/set/list), status queries
- [x] Does NOT advertise inject/output/stream capabilities (honest boundary)
- [x] `termlink::endpoint` public API in termlink-session crate for library use
- [x] Session cleans up (socket + JSON sidecar) on Ctrl+C / SIGHUP / drop
- [x] Unit tests: self-registration, event emit/poll, KV, cleanup on drop (4 tests)
- [x] Integration: hub discovers self-registered session (uses same registration path as register --shell, tested via existing hub discovery tests)

## Verification

/Users/dimidev32/.cargo/bin/cargo test -p termlink-session endpoint 2>&1 | grep "test result: ok"
grep -q "self" crates/termlink-cli/src/commands/session.rs

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

### 2026-03-24T10:47:34Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-263-register---self-event-only-endpoint-for-.md
- **Context:** Initial task creation
