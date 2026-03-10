---
id: T-011
name: "IT-008: Distributed topology — cross-machine, containers, NAT"
description: >
  Broker federation, NAT traversal, container networking, SSH tunneling

status: captured
workflow_type: inception
owner: agent
horizon: later
tags: []
components: []
related_tasks: []
created: 2026-03-08T14:19:49Z
last_update: 2026-03-08T14:19:49Z
date_finished: null
---

# T-011: IT-008: Distributed topology — cross-machine, containers, NAT

## Problem Statement

TermLink is currently local-only (Unix sockets). For multi-machine agent coordination (dev laptop + cloud VMs, container orchestration, CI workers), sessions need to communicate across network boundaries. The architecture analysis (docs/reports/reflection-result-arch.md) noted the session crate is hardcoded to tokio + Unix sockets with no transport abstraction. This inception explores what distributed topology looks like: broker federation, NAT traversal, container networking, and whether SSH tunneling is sufficient or a native TCP/TLS transport is needed.

## Assumptions

- A1: SSH tunneling over Unix sockets is sufficient for 90% of cross-machine use cases
- A2: Container networking (Docker bridge, Kubernetes pod networking) can use TCP sockets without NAT traversal
- A3: A trait-based transport abstraction (T-073) is a prerequisite for distributed topology
- A4: Hub federation (multiple hubs peering) is more complex than hub-spoke (single hub, remote sessions)

## Exploration Plan

1. **Research (1h):** Survey existing approaches — MCP over SSH, tmux remote sessions, VS Code remote development
2. **Spike 1 (1h):** Test Unix socket forwarding over SSH tunnel — does the TermLink protocol work transparently?
3. **Spike 2 (1h):** Test TermLink in Docker containers — can two containers communicate via TCP socket?
4. **Design (2h):** Draft topology options: SSH tunneling vs. native TCP/TLS vs. broker federation

## Technical Constraints

- NAT traversal requires relay servers or STUN/TURN — significant infrastructure
- TLS certificate management adds operational complexity (CA, rotation, revocation)
- Latency: cross-machine RPC adds 1-100ms vs. <1ms for local Unix sockets
- Transport abstraction (T-073) must land before any distributed transport implementation

## Scope Fence

**IN scope:** Transport options analysis, SSH tunneling feasibility, container networking, hub federation design.
**OUT of scope:** Implementation of TCP/TLS transport (that's a build task after inception), cloud deployment, auto-discovery across networks, zero-trust networking.

## Acceptance Criteria

- [ ] Problem statement validated
- [ ] Assumptions tested
- [ ] Go/No-Go decision made

## Go/No-Go Criteria

**GO if:**
- SSH tunneling works transparently with existing protocol (zero code changes)
- A clear topology model (hub-spoke or federated) emerges with manageable complexity
- Transport abstraction (T-073) is feasible and doesn't require protocol changes

**NO-GO if:**
- Cross-machine use cases are too niche to justify the complexity
- SSH tunneling handles all real-world scenarios adequately (no native transport needed)

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
