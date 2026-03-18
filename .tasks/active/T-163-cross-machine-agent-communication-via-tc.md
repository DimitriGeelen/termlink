---
id: T-163
name: "Cross-machine agent communication via TCP hub"
description: >
  Inception: Can two Claude Code agents on different LAN machines communicate
  in near real-time via TermLink TCP hub, with bidirectional messaging,
  file transfer, and appropriate security?

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: [remote-access, tcp, security, cross-machine]
components: []
related_tasks: [T-145, T-146, T-147, T-155, T-157]
created: 2026-03-18T09:42:29Z
last_update: 2026-03-18T22:20:24Z
date_finished: null
---

# T-163: Cross-machine agent communication via TCP hub

## Problem Statement

Two Claude Code agents run on different LAN machines (e.g., TermLink project on machine A,
framework project on machine B). Currently they can only coordinate through git (push/pull) —
high latency, no real-time signaling, no direct file transfer.

We want near real-time bidirectional communication: agent A sends a request to agent B,
agent B processes it and responds, with the ability to transfer files alongside messages.
This must be secure — an open TCP port on LAN should not expose agent I/O to unauthorized users.

## Assumptions

- A1: TCP hub already bridges sessions across machines (built in T-145/T-146/T-147)
- A2: Events (emit/watch) work over TCP hub for real-time messaging
- A3: Session discovery works cross-machine (hybrid local+remote in hub)
- A4: Capability tokens (T-079) can secure cross-machine access
- A5: File transfer can be layered on top of events or data plane
- A6: Agents can be taught a message protocol via CLAUDE.md / skills

## Exploration Plan

1. [x] Research existing TCP hub capabilities and security model (3 agents)
2. [ ] Research file transfer approaches (agent)
3. [ ] Research agent message protocol design (agent)
4. [ ] Assess security gaps for LAN exposure
5. [ ] Human dialogue: review findings, validate approach
6. [ ] Go/No-Go decision

## Technical Constraints

- **Network:** Same LAN, machines can reach each other by IP
- **Firewall:** Must work behind typical home/office router (no port forwarding to internet)
- **Security:** TCP port on LAN is accessible to all LAN devices — need auth
- **Latency:** Sub-second for messages, seconds acceptable for file transfer
- **File size:** Mostly small files (task files, specs, configs <100KB), occasionally larger (source files, reports)
- **Existing auth:** Capability tokens (HMAC-SHA256) exist but are session-scoped, not yet wired to TCP transport

## Scope Fence

**IN:**
- Validating TCP hub works for agent-to-agent messaging
- Designing a message protocol (request/response/status)
- Evaluating file transfer approaches
- Assessing security model for LAN exposure
- Identifying what needs to be built vs what exists

**OUT:**
- Internet/WAN connectivity (LAN only for now)
- Building the full implementation (separate build tasks)
- Multi-tenant auth (single-user, two machines)
- Encryption at rest

## Acceptance Criteria

- [x] Problem statement validated
- [x] Assumptions A1-A6 assessed
- [x] Security model for LAN TCP hub evaluated
- [x] File transfer approach recommended
- [x] Agent message protocol sketched
- [x] Research artifact written (docs/reports/T-163-*.md)
- [x] Go/No-Go decision made

## Go/No-Go Criteria

**GO if:**
- TCP hub events work cross-machine (A1+A2)
- Security can be added without major protocol changes (tokens or TLS)
- File transfer is feasible without rewriting data plane
- Total build effort bounded (≤3 sessions)

**NO-GO if:**
- TCP hub has fundamental issues preventing reliable messaging
- Security requires protocol redesign
- File transfer needs major new infrastructure
- Simpler alternative (SSH+rsync+git) meets the need adequately

## Verification

test -f docs/reports/T-163-cross-machine-agent-communication.md

## Decisions

**Decision**: GO

**Rationale**: TCP hub infrastructure 80% ready. Security hardening (token enforcement + TLS) is non-negotiable prerequisite but additive (2 sessions). Events over TCP need validation but code path exists. File transfer via base64 chunked events feasible. Total effort ~5.5 sessions, each phase independently useful.

**Date**: 2026-03-18T10:07:49Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-03-18T10:07:49Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** TCP hub infrastructure 80% ready. Security hardening (token enforcement + TLS) is non-negotiable prerequisite but additive (2 sessions). Events over TCP need validation but code path exists. File transfer via base64 chunked events feasible. Total effort ~5.5 sessions, each phase independently useful.
