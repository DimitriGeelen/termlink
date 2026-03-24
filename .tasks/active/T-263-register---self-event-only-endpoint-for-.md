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
- [ ] `--self` flag on `register` CLI command (mutually exclusive with `--shell`)
- [ ] Self-registered session runs RPC server in current process (Unix socket, hub discovery)
- [ ] Capabilities: events (emit/poll), KV (get/set/list), status queries
- [ ] Does NOT advertise inject/output/stream capabilities (honest boundary)
- [ ] `termlink::endpoint` public API in termlink-session crate for library use
- [ ] Session cleans up (socket + JSON sidecar) on Ctrl+C / SIGHUP / drop
- [ ] Unit tests: self-registration, event emit/poll through self-registered session, cleanup on drop
- [ ] Integration: hub discovers self-registered session, emit-to delivers events to it

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
