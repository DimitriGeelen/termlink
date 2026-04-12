---
id: T-690
name: "Event subscription — push-based event delivery to eliminate polling in dispatch/request/watch"
description: >
  Inception: Event subscription — push-based event delivery to eliminate polling in dispatch/request/watch

status: work-completed
workflow_type: inception
owner: human
horizon: now
tags: []
components: [crates/termlink-protocol/src/control.rs, crates/termlink-session/src/auth.rs, crates/termlink-session/src/events.rs, crates/termlink-session/src/handler.rs, crates/termlink-session/src/server.rs]
related_tasks: []
created: 2026-03-28T23:38:01Z
last_update: 2026-04-12T09:14:50Z
date_finished: 2026-04-12T08:43:25Z
---

# T-690: Event subscription — push-based event delivery to eliminate polling in dispatch/request/watch

## Problem Statement

TermLink's event system is entirely poll-based. Every consumer (`watch`, `wait`, `collect`, `dispatch`, `request`) runs a tokio::select! loop with fixed sleep intervals (250-500ms), making RPC calls to pull events. This creates a latency floor, wastes work on idle polls, and risks event loss via ring buffer overflow between polls. As orchestration use cases grow (dispatch, multi-agent coordination), the polling overhead scales linearly with session count.

## Assumptions

1. The data plane broadcast pattern (tokio::sync::broadcast for PTY streaming) is transferable to structured events
2. Adding broadcast::Sender to EventBus alongside the ring buffer is backward compatible
3. Streaming RPC responses can be delivered over the existing Unix socket control plane
4. Hub subscription aggregation (subscribe to N sessions, republish) is feasible without excessive connection overhead

## Exploration Plan

1. **Spike 1: EventBus broadcast** (30min) — Add broadcast::Sender<Event> to EventBus, verify emit() delivers to both ring buffer and channel. Unit test subscription receive.
2. **Spike 2: Streaming RPC** (1hr) — Prototype `event.subscribe` RPC that holds connection and streams events as newline-delimited JSON. Test with netcat/socat.
3. **Spike 3: CLI integration** (30min) — Modify `watch` command to use subscription with poll fallback. Measure latency improvement.

## Technical Constraints

- Unix socket control plane uses request-response JSON-RPC 2.0 — streaming requires protocol extension (newline-delimited responses on single connection, or upgrade to separate subscription connection)
- tokio broadcast channel drops messages when receiver is lagged — need lag handling strategy
- Hub aggregation requires one subscription per monitored session — connection count = O(sessions)
- TCP remote connections add latency; subscription keepalive needed for remote hub

## Scope Fence

**IN scope:**
- EventBus internal broadcast mechanism
- `event.subscribe` RPC method design
- CLI `watch` command as first consumer
- Backward compatibility assessment

**OUT of scope:**
- Hub subscription aggregation (separate task if GO)
- MCP subscription tools (separate task)
- Data plane event multiplexing (Option B rejected)
- Remote TCP subscription (separate task)

## Acceptance Criteria

### Agent
- [x] Problem statement validated
- [x] Assumptions tested
- [x] Recommendation written with rationale

### Human
- [ ] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Read the research artifact and recommendation in this task
  2. Evaluate go/no-go criteria against findings
  3. Run: `cd /opt/999-Agentic-Engineering-Framework && bin/fw inception decide T-XXX go|no-go --rationale "your rationale"`
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

**GO if:**
- EventBus broadcast::Sender works alongside ring buffer without breaking existing poll()
- Streaming responses can be delivered over Unix socket without protocol redesign
- Latency improvement is measurable (>10x vs poll baseline)

**NO-GO if:**
- JSON-RPC 2.0 request-response model cannot be extended for streaming without breaking clients
- broadcast channel lag handling adds more complexity than polling eliminates
- Connection lifetime management for subscriptions is unacceptable overhead

## Verification

# Research artifact exists
test -f docs/reports/T-690-event-subscription-research.md

## Decisions

**Decision**: GO

**Rationale**: Recommendation: GO

Rationale: Hybrid approach (broadcast channel + long-poll RPC) achieves near-zero latency without protocol changes. All 3 go criteria met across 2 spikes. Backward compatible wi...

**Date**: 2026-04-12T08:43:24Z

## Recommendation

**Recommendation:** GO

**Rationale:** Hybrid approach (broadcast channel + long-poll RPC) achieves near-zero latency without protocol changes. All 3 go criteria met across 2 spikes. Backward compatible with existing poll consumers.

## Decision

**Decision**: GO

**Rationale**: Recommendation: GO

Rationale: Hybrid approach (broadcast channel + long-poll RPC) achieves near-zero latency without protocol changes. All 3 go criteria met across 2 spikes. Backward compatible wi...

**Date**: 2026-04-12T08:43:24Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-03-30T11:43:30Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-04-12T08:43:24Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Recommendation: GO

Rationale: Hybrid approach (broadcast channel + long-poll RPC) achieves near-zero latency without protocol changes. All 3 go criteria met across 2 spikes. Backward compatible wi...

### 2026-04-12T08:43:25Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
