---
id: T-256
name: "Inception: True push messaging — emit-to-target RPC for zero-poll agent communication"
description: >
  Inception: Should TermLink add an emit-to-target RPC so workers can push events directly
  to the orchestrator's event bus? Currently all event consumption is poll-based (collect/watch).
  Option B (collect fan-in) ships as T-257 with no code changes. This inception evaluates whether
  true push (Option A) is worth the protocol change.

status: started-work
workflow_type: build
build_phase: true
owner: agent
horizon: now
tags: [orchestration, protocol, events]
components: []
related_tasks: [T-257, T-233, T-247]
created: 2026-03-23T22:15:21Z
last_update: 2026-03-24T09:51:14Z
date_finished: null
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

### Inception (done)
- [x] Problem statement validated
- [x] Assumptions A1-A4 tested with evidence
- [x] Go/No-Go decision made

### Agent
- [ ] `EVENT_EMIT_TO` constant in `control.rs`
- [ ] `handle_event_emit_to` hub RPC handler — resolve target session, forward `event.emit`, return result
- [ ] Backward-compatible: sender field included in emitted event payload so target knows origin
- [ ] `termlink event emit-to <target> <topic> [--payload JSON] [--from SESSION]` CLI command
- [ ] Top-level `termlink emit-to` alias
- [ ] Hub tests: target resolution, emit forwarding, unknown target error, sender enrichment
- [ ] Fabric card registered

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
/Users/dimidev32/.cargo/bin/cargo test -p termlink-hub emit_to 2>&1 | grep "test result: ok"
grep -q "EVENT_EMIT_TO" crates/termlink-protocol/src/control.rs

## Decisions

### 2026-03-24T07:08:21Z — Original NO-GO (REVERSED)
- **Decision:** NO-GO (now overridden)
- **Original rationale:** T-257 collect fan-in solved the immediate problem. 500ms poll latency negligible for minute-long agent tasks. Ring buffer has 34x headroom at current worker counts.
- **Research:** `docs/reports/T-256-inception-decision.md` + 3 mesh agent reports (`T-256-q1-primitives.md`, `T-256-q2-dispatch.md`, `T-256-q3-execution-model.md`). Q1 cataloged 15+ messaging commands, identified 6 gaps (G1/G5 "no push notification" rated High severity). Q2 documented the dormant `bus-handler.sh` inbox designed for push but never activated. Q3 found `collect --count N` as background Bash = ~800 tokens vs polling loop = ~10K-18K tokens.
- **Valid findings preserved:** T-257 collect fan-in works today and is a valid Layer 1. A3 (backward-compatible) LIKELY VALID — optional `target` field on emit. The protocol change is non-trivial but bounded (~new handler in control.rs + target socket resolution).

### 2026-03-24T08:05:00Z — Reversed to GO (human decision)
- **Chose:** GO — build emit-to-target push RPC
- **Why:** Push messaging is important. This was the user's original feature request, born from real frustration with the fire-and-forget polling model. Push enables communication patterns that poll cannot support: real-time bidirectional dialogue between agents (the negotiation protocol T-240 needs this), streaming progress updates without poll overhead, interactive correction mid-task. As agent count scales beyond 10+ workers, poll-based collection becomes a throughput bottleneck (hub polls each worker at 500ms intervals = O(N) poll load). Push is O(1) per event — worker emits directly to orchestrator's bus. The research confirmed the protocol supports it (A3 backward-compatible) and the design is bounded. T-256 is also foundational for the T-233 architecture — the negotiation protocol (T-240) assumes agents can talk directly to each other, not just emit to their own bus.
- **Rejected:** Original NO-GO — dismissed the user's feature request by framing current workaround (collect) as sufficient. Evaluated latency only for current scale (minute-long tasks, 3-10 workers) rather than the architecture being built.

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

### 2026-03-24T08:05:00Z — reopened [human decision]
- **Action:** NO-GO reversed to GO by human
- **Reason:** Push messaging is important. Enables real-time bidirectional agent communication, required by negotiation protocol (T-240), scales better than poll-based collection. User's original feature request.
- **Context:** T-258 context amnesia investigation revealed NO-GO was based on missing architectural context + dismissing user's feature request

### 2026-03-24T09:51:14Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
