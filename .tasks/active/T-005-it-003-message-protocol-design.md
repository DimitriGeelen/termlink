---
id: T-005
name: "IT-003: Message protocol design"
description: >
  Design wire format, message types, envelope fields, versioning

status: started-work
workflow_type: inception
owner: agent
horizon: next
tags: []
components: []
related_tasks: []
created: 2026-03-08T14:19:33Z
last_update: 2026-03-08T15:18:06Z
date_finished: null
---

# T-005: IT-003: Message protocol design

## Problem Statement

What is the wire format for TermLink messages? T-003 established a message bus with control/data plane split. T-004 determines which protocols to build on. This task designs the actual message types, envelope fields, framing, encoding, and versioning. This is the contract between all participants — get it wrong and everything downstream suffers.

## Assumptions

- A-001: JSON-RPC 2.0 is adequate for control plane messages (latency acceptable for non-streaming ops)
- A-002: Length-prefixed binary framing is needed for data plane (streaming output, raw injection)
- A-003: A small set of message types (<10) covers the core use cases
- A-004: Protocol versioning can be handled via envelope field without breaking changes

## Exploration Plan

1. **Message type taxonomy** (30 min) — Map use cases from T-002/T-003 to concrete message types. What's the minimum viable set?
2. **Envelope design** (20 min) — Required fields (sender, target, id, type, timestamp). Optional fields. Size overhead analysis.
3. **Framing analysis** (20 min) — Compare: newline-delimited JSON, length-prefixed frames, MessagePack, Protobuf. Evaluate against D1-D4.
4. **Special key encoding** (15 min) — How to represent Ctrl+C, arrow keys, escape sequences in injection messages.
5. **Versioning strategy** (15 min) — How the protocol evolves. Backward compatibility approach.

## Technical Constraints

- Control plane: JSON-RPC 2.0 compatible (T-003 decision)
- Data plane: binary-safe, low-latency, length-prefixed (T-003 decision)
- Must handle UTF-8 text AND raw terminal escape sequences
- Messages must be self-describing (type field) for routing
- Must support correlation (request/response pairing)

## Scope Fence

**IN:** Wire format, message types, envelope fields, framing, encoding, versioning, special key representation.
**OUT:** Transport selection (T-004 covers protocol choice). Session discovery (T-006). Security/auth headers (T-008). Concurrency semantics (T-009).

## Acceptance Criteria

- [ ] Problem statement validated
- [ ] Assumptions tested
- [ ] Go/No-Go decision made
- [ ] Message type catalog produced
- [ ] Envelope format specified
- [ ] Framing decision made with rationale
- [ ] Research artifact committed to docs/reports/

## Go/No-Go Criteria

**GO if:**
- A coherent protocol design emerges that covers all T-003 use cases
- Framing choice satisfies D1 (antifragile: parseable even with corruption) and D2 (reliable: deterministic)
- Message set is minimal but extensible

**NO-GO if:**
- Message types proliferate beyond manageable complexity (>15 types for v1)
- No framing approach satisfies both binary safety and human debuggability

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

### 2026-03-08T15:18:06Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
