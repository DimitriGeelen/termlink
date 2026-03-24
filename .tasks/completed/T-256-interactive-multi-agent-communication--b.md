---
id: T-256
name: "Inception: True push messaging — emit-to-target RPC for zero-poll agent communication"
description: >
  Inception: Should TermLink add an emit-to-target RPC so workers can push events directly
  to the orchestrator's event bus? Currently all event consumption is poll-based (collect/watch).
  Option B (collect fan-in) ships as T-257 with no code changes. This inception evaluates whether
  true push (Option A) is worth the protocol change.

status: work-completed
workflow_type: inception
owner: agent
horizon: next
tags: [orchestration, protocol, events]
components: []
related_tasks: [T-257, T-233, T-247]
created: 2026-03-23T22:15:21Z
last_update: 2026-03-24T07:08:21Z
date_finished: 2026-03-24T07:08:21Z
---

# T-256: Inception — True push messaging (emit-to-target RPC)

## Problem Statement

Workers can only emit events to their *own* event bus. There is no `emit-to <target>` RPC.
The orchestrator must always poll (via `event.collect`) to discover worker results. While
Option B (collect-based fan-in via hub, T-257) eliminates polling at the *agent* level, the
hub still polls each worker session at 500ms intervals. For high-throughput or latency-sensitive
scenarios, true push would eliminate this entirely.

**For whom:** Framework orchestrators dispatching 3-10+ concurrent workers.
**Why now:** T-257 (Option B) ships the convention; this inception decides if the protocol
investment for true push is justified or if collect-based fan-in is good enough.

## Assumptions

- A1: 500ms hub poll latency is noticeable/problematic in real multi-agent workflows
- A2: Ring buffer overflow (1024 events) is a real risk under high fan-in
- A3: The protocol change (emit-to-target) can be made backward-compatible
- A4: True push reduces hub CPU load compared to continuous polling

## Exploration Plan

1. **Benchmark A1:** Measure collect latency in a 5-worker dispatch (real Claude sessions). Is 500ms visible? (1 hour)
2. **Stress-test A2:** Emit 2000+ events rapidly to one session, measure gap detection / data loss (30 min)
3. **Prototype A3:** Spike an `event.emit_to` handler in session — worker connects to target socket and emits directly (2 hours)
4. **Measure A4:** Compare hub CPU during 5-worker collect polling vs idle (30 min)

## Technical Constraints

- Protocol change must be backward-compatible (old clients ignore new `target` field)
- Session socket access: worker needs to know orchestrator's socket path (discoverable via hub)
- Security: emit-to-target could be used to spam a session's event bus — may need rate limiting or auth

## Scope Fence

**IN scope:** emit-to-target RPC design, backward compat analysis, latency/throughput benchmarks
**OUT of scope:** subscription model (persistent push channels), data plane changes, cross-machine push (TCP), `fw dispatch` CLI (that's T-257)

## Acceptance Criteria

- [x] Problem statement validated
- [x] Assumptions A1-A4 tested with evidence
- [x] Go/No-Go decision made

## Go/No-Go Criteria

**GO if:**
- 500ms poll latency measurably impacts orchestration throughput with 5+ workers
- Ring buffer overflow occurs in realistic scenarios (not just synthetic stress)
- Prototype shows <50ms push latency and backward-compatible wire format

**NO-GO if:**
- Collect-based fan-in (T-257) handles all real-world scenarios within acceptable latency
- Ring buffer overflow requires unrealistic event rates (>100/sec sustained)
- Protocol change introduces backward-compatibility risk that outweighs latency gain

## Verification

## Decisions

**Decision**: NO-GO

**Rationale**: T-257 collect fan-in solved the problem. 500ms poll latency negligible for minute-long agent tasks. Ring buffer has 34x headroom.

**Date**: 2026-03-24T07:08:21Z
## Decision

**Decision**: NO-GO

**Rationale**: T-257 collect fan-in solved the problem. 500ms poll latency negligible for minute-long agent tasks. Ring buffer has 34x headroom.

**Date**: 2026-03-24T07:08:21Z

## Updates

### 2026-03-23T22:15:21Z — task-created [task-create-agent]
- **Action:** Created inception task for interactive multi-agent communication

### 2026-03-23T23:35:00Z — research complete [3 TermLink mesh agents]
- **Action:** Q1 (primitives), Q2 (dispatch architecture), Q3 (execution model) research delivered
- **Output:** docs/reports/T-256-q1-primitives.md, T-256-q2-dispatch.md, T-256-q3-execution-model.md
- **Key finding:** Collect-based fan-in works today (Option B). True push needs protocol change (Option A).

### 2026-03-24T07:08:21Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** NO-GO
- **Rationale:** T-257 collect fan-in solved the problem. 500ms poll latency negligible for minute-long agent tasks. Ring buffer has 34x headroom.

### 2026-03-24T07:08:21Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: NO-GO
