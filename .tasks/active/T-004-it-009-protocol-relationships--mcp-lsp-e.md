---
id: T-004
name: "IT-009: Protocol relationships — MCP, LSP, existing standards"
description: >
  Investigate whether to extend MCP/LSP or build a new protocol

status: started-work
workflow_type: inception
owner: agent
horizon: next
tags: []
components: []
related_tasks: []
created: 2026-03-08T14:19:29Z
last_update: 2026-03-08T15:18:45Z
date_finished: null
---

# T-004: IT-009: Protocol relationships — MCP, LSP, existing standards

## Problem Statement

Are we reinventing the wheel? T-003 decided on "message bus + injection adapter with control/data plane split." Before designing our own protocol (T-005), we must determine whether to extend MCP, compose with existing protocols (LSP, gRPC, NATS, ZeroMQ, D-Bus), or build from scratch. D4 (portability) demands we prefer standards.

## Assumptions

- A-001: MCP can serve as the control plane for terminal session management
- A-002: MCP's limitations (no true streaming, JSON-RPC overhead) require a separate data plane
- A-003: No existing protocol covers both control and data plane needs for terminal communication
- A-004: The MCP ecosystem is mature enough to build on (stable spec, multiple implementations)

## Exploration Plan

1. **MCP deep dive** (30 min) — Current spec capabilities, transport options, extension mechanisms, ecosystem maturity
2. **Alternative protocol analysis** (30 min) — gRPC, NATS, ZeroMQ, D-Bus: what each brings, what each lacks
3. **Composition patterns** (20 min) — How could MCP + another protocol compose?
4. **Decision** (10 min) — Extend MCP, compose, or build new?

## Technical Constraints

- Must support macOS and Linux (D4 portability)
- Must support local (Unix socket) and remote (TCP/TLS) transport
- Control plane must be JSON-RPC compatible (T-003 decision)
- Data plane must be binary-safe and low-latency (T-003 decision)
- Must not require heavy infrastructure (no mandatory broker services)

## Scope Fence

**IN:** Protocol selection for control plane and data plane. Integration strategy with MCP ecosystem. Comparison matrix.
**OUT:** Detailed wire format design (T-005). Security model (T-008). Distributed topology (T-011).

## Acceptance Criteria

- [ ] Problem statement validated
- [ ] Assumptions tested
- [ ] Go/No-Go decision made
- [ ] Protocol comparison matrix produced
- [ ] Integration recommendation documented
- [ ] Research artifact committed to docs/reports/

## Go/No-Go Criteria

**GO if:**
- A clear protocol strategy emerges (extend, compose, or build) with evidence
- The chosen approach satisfies all four constitutional directives
- At least one viable composition pattern identified

**NO-GO if:**
- No existing protocol covers >50% of requirements (full custom build with no standards leverage)
- MCP spec is too unstable or immature to build on

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

### 2026-03-08T15:18:45Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
