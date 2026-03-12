---
id: T-009
name: "IT-005: Concurrency, ordering, and backpressure"
description: >
  Multiple senders, message ordering, queue management, typing races

status: started-work
workflow_type: inception
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-08T14:19:43Z
last_update: 2026-03-12T19:02:57Z
date_finished: null
---

# T-009: IT-005: Concurrency, ordering, and backpressure

## Problem Statement

TermLink's event system uses polling with cursor-based pagination. Under multi-agent load (e.g., 10 specialists polling simultaneously in L6 tests), there are no guarantees on event ordering, no backpressure mechanism, and no protection against slow consumers falling behind. The reflection fleet test coverage analysis (docs/reports/reflection-result-testcov.md) found zero tests for concurrent client connections. The event schema report (docs/reports/reflection-result-evschema.md) noted that free-form polling will miss events under load. This inception explores whether the current model is adequate or needs upgrading to push-based delivery.

## Assumptions

- A1: Current polling model handles up to ~5 concurrent agents without event loss
- A2: Event ordering within a single session is guaranteed by sequential sequence numbers
- A3: Cross-session event ordering (via hub broadcast) is not guaranteed and doesn't need to be
- A4: Backpressure is unnecessary for local deployment (bounded by local CPU/memory)

## Exploration Plan

1. **Spike 1 (1h):** Stress test — 10 concurrent pollers on one session, measure event loss/ordering
2. **Spike 2 (1h):** Measure event throughput ceiling (events/sec before polling falls behind)
3. **Research (30m):** Compare polling vs. push (WebSocket/SSE) for event delivery trade-offs
4. **Design (1h):** If polling is insufficient, draft push-based delivery design with backpressure

## Technical Constraints

- Current event store is append-only in-memory Vec — no persistence, no compaction
- Polling interval in watchers is 2 seconds — latency floor for event detection
- Hub broadcasts events to all sessions — fan-out amplifies under load

## Scope Fence

**IN scope:** Event ordering guarantees, concurrent poller behavior, backpressure design, event delivery reliability.
**OUT of scope:** Distributed event ordering across machines (T-011), event persistence/WAL, exactly-once delivery semantics.

## Acceptance Criteria

- [ ] Problem statement validated
- [ ] Assumptions tested
- [ ] Go/No-Go decision made

## Go/No-Go Criteria

**GO if:**
- Stress test shows event loss or ordering violations under 10+ concurrent pollers
- Push-based delivery can be added without breaking existing polling API (additive change)

**NO-GO if:**
- Polling handles 10+ concurrent agents reliably with no event loss
- Backpressure is unnecessary because local event throughput never exceeds consumer capacity

## Verification

<!-- Shell commands that MUST pass before work-completed. One per line.
     Lines starting with # are comments. Empty lines ignored.
     The completion gate runs each command — if any exits non-zero, completion is blocked.
     For inception tasks, verification is often not needed (decisions, not code).
-->

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Decision

<!-- Filled at completion via: fw inception decide T-XXX go|no-go --rationale "..." -->

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-03-12T18:58:11Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-12T19:02:57Z — status-update [task-update-agent]
- **Change:** horizon: later → now
