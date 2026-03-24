---
id: T-262
name: "Inception: Attach-self — register existing shell as TermLink endpoint"
description: >
  Pickup from fw-agent T-600. Add termlink attach command to wrap current shell as TermLink endpoint, discoverable via hub. Key questions: can register already do this, lifecycle on shell exit, SSH forwarding interaction, security model.

status: work-completed
workflow_type: inception
owner: agent
horizon: next
tags: [pickup, cli]
components: []
related_tasks: []
created: 2026-03-24T09:27:26Z
last_update: 2026-03-24T10:46:51Z
date_finished: 2026-03-24T10:46:51Z
---

# T-262: Inception: Attach-self — register existing shell as TermLink endpoint

## Problem Statement

Users SSH into remote machines and want to make that shell a TermLink endpoint — discoverable via hub, able to send/receive events. Currently `register --shell` spawns a **new** PTY; there's no way to register the **current** shell. Use case: agent on .107 runs `termlink attach`, local orchestrator discovers it via hub and communicates via events.

**For whom:** Framework agents on remote machines needing cross-machine communication.
**Why now:** T-233 orchestration system (emit-to, negotiation, collect) assumes sessions exist. Remote agents need a lightweight way to become endpoints.

## Assumptions

- A1: The primary need is event-based communication (emit/poll/collect), not PTY inject/output
- A2: PTY master FD is not accessible from a child process (SSH owns it), making real inject impossible
- A3: `register` can be extended with a `--self` flag rather than needing a new command
- A4: Event-only endpoints are sufficient for the agent communication use case

## Exploration Plan

1. **Validate A1:** Review pickup message use cases — do any require inject/output? (15 min)
2. **Validate A2:** Research PTY master FD accessibility from child processes on Linux/macOS (15 min)
3. **Validate A3:** Read `register` code path to confirm `--self` flag is a clean extension (15 min)
4. **Design:** Sketch the `--self` code path in session.rs (30 min)

## Technical Constraints

- PTY master FD owned by SSH daemon / terminal emulator — not accessible to child processes
- Shell integration hooks (`PROMPT_COMMAND`, `precmd`) only fire at prompt boundaries — not real-time
- Unix socket + RPC server can run in a background tokio task within the same process
- Cross-machine requires hub TCP relay (already built)

## Scope Fence

**IN scope:** `register --self` for event-only endpoint, design analysis of 6 options, go/no-go
**OUT of scope:** PTY interposition, shell integration hooks, inject support for self-registered sessions

## Acceptance Criteria

- [x] Problem statement validated
- [x] Assumptions A1-A4 tested via code analysis
- [x] Go/No-Go decision made

## Go/No-Go Criteria

**GO if:**
- Event-only endpoint covers the cross-machine agent communication use case
- Implementation is bounded (~50 lines, reuses existing RPC server)
- No PTY complexity needed

**NO-GO if:**
- Use case requires real-time inject/output into existing shell
- PTY interposition is the only viable approach (high complexity, fragile)

## Verification

<!-- Shell commands that MUST pass before work-completed. One per line.
     Lines starting with # are comments. Empty lines ignored.
     The completion gate runs each command — if any exits non-zero, completion is blocked.
     For inception tasks, verification is often not needed (decisions, not code).
-->

## Decisions

**Decision**: GO

**Rationale**: GO for reframed deliverable: library API + register --self CLI flag. Any process becomes a TermLink endpoint (events, KV, discovery) without TermLink owning it. No PTY/inject — honest capability boundary. Serves daemons, mid-session agents, any Rust binary wanting mesh presence.

**Date**: 2026-03-24T10:46:51Z
## Decision

**Decision**: GO

**Rationale**: GO for reframed deliverable: library API + register --self CLI flag. Any process becomes a TermLink endpoint (events, KV, discovery) without TermLink owning it. No PTY/inject — honest capability boundary. Serves daemons, mid-session agents, any Rust binary wanting mesh presence.

**Date**: 2026-03-24T10:46:51Z

## Updates

### 2026-03-24T09:27:26Z — task-created [pickup from fw-agent T-600 on .107]
- **Source:** File transfer via TermLink (`termlink-pickup-003-attach-self.md`)
- **Pickup message:** Add `termlink attach` to wrap current shell as TermLink endpoint. Gap: `register` is for spawned processes, not existing interactive shells. Use case: SSH into remote, run `termlink attach`, local agent connects via hub.

### 2026-03-24T10:01:08Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-24T10:46:51Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** GO for reframed deliverable: library API + register --self CLI flag. Any process becomes a TermLink endpoint (events, KV, discovery) without TermLink owning it. No PTY/inject — honest capability boundary. Serves daemons, mid-session agents, any Rust binary wanting mesh presence.

### 2026-03-24T10:46:51Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
