---
id: T-073
name: "Transport abstraction — trait-based transport in protocol crate"
description: >
  Session crate couples to tokio and Unix sockets. Inception: trait-based transport abstraction to enable TCP, QUIC, or other transports.

status: captured
workflow_type: inception
owner: agent
horizon: later
tags: []
components: []
related_tasks: []
created: 2026-03-10T08:44:51Z
last_update: 2026-03-10T08:44:51Z
date_finished: null
---

# T-073: Transport abstraction — trait-based transport in protocol crate

## Problem Statement

Architectural coupling found by reflection fleet architecture agent. Session crate couples to tokio + Unix sockets with no trait-based transport abstraction. See [docs/reports/reflection-result-arch.md].

## Assumptions

- A1: A `Transport` trait with `connect()`, `accept()`, `send()`, `recv()` methods can abstract over Unix sockets, TCP, and QUIC
- A2: The trait can be defined in `termlink-protocol` without pulling in tokio as a dependency (use `async-trait` or `std::future`)
- A3: Existing Unix socket implementation can be wrapped to implement the trait without performance regression
- A4: Transport selection can be configuration-driven (e.g., `--transport unix` vs. `--transport tcp`)

## Exploration Plan

1. **Spike 1 (1h):** Draft `Transport` trait API — what methods are needed? Look at how `tower::Service` and `hyper` abstract transports
2. **Spike 2 (1h):** Prototype Unix socket implementation behind the trait — measure overhead vs. direct calls
3. **Research (30m):** Survey Rust transport abstraction patterns — `tokio::net` traits, `async-net`, custom approaches
4. **Design (1h):** Finalize trait design, decide where it lives (protocol vs. new crate), plan migration path

## Technical Constraints

- `termlink-protocol` currently has zero runtime dependencies (just serde) — adding tokio would be a significant change
- The trait must support both control plane (JSON-RPC messages) and data plane (binary frames)
- `libc` dependency in session crate (for signal handling, PTY) is separate from transport

## Scope Fence

**IN scope:** Trait definition, Unix socket adapter, API design for TCP adapter (stub, not implemented).
**OUT of scope:** TCP/TLS implementation (build task after inception GO), QUIC support, transport encryption.

## Acceptance Criteria

- [ ] Problem statement validated
- [ ] Assumptions tested
- [ ] Go/No-Go decision made

## Go/No-Go Criteria

**GO if:**
- Trait API is clean and covers both control and data plane needs
- Unix socket adapter adds <1% overhead vs. direct implementation
- Trait can live in protocol crate without adding heavy dependencies

**NO-GO if:**
- Transport abstraction requires leaking async runtime details into the protocol crate
- The overhead of trait dispatch is measurable in latency-sensitive paths
- A simpler approach (compile-time feature flags) covers the same use cases

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
