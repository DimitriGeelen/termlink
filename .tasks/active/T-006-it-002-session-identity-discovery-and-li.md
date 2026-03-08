---
id: T-006
name: "IT-002: Session identity, discovery, and lifecycle"
description: >
  Design how sessions find each other — naming, registration, discovery, liveness

status: started-work
workflow_type: inception
owner: agent
horizon: next
tags: []
components: []
related_tasks: []
created: 2026-03-08T14:19:34Z
last_update: 2026-03-08T15:18:46Z
date_finished: null
---

# T-006: IT-002: Session identity, discovery, and lifecycle

## Problem Statement

How do TermLink sessions find each other, and what happens when they appear/disappear? Without discovery, you need hardcoded addresses. Without lifecycle management, you get stale references and silent delivery failures. This is foundational — the message protocol (T-005) needs addressable endpoints, and every higher-level feature depends on reliable session identity.

## Assumptions

- A-001: Sessions need human-readable names (not just UUIDs) for usability (D3)
- A-002: Automatic registration on session start is preferable to explicit opt-in
- A-003: Filesystem-based discovery (XDG_RUNTIME_DIR / /tmp) is sufficient for local operation
- A-004: Session liveness can be determined by socket connectivity (no heartbeat needed for v1)
- A-005: Sessions inside containers/SSH need explicit bridge configuration (not auto-discovered)

## Exploration Plan

1. **Naming scheme** (20 min) — Human-readable names, UUIDs, role-based, composite identifiers. Prior art comparison.
2. **Registration protocol** (20 min) — Auto vs explicit. What gets registered. Where stored.
3. **Discovery mechanisms** (25 min) — Filesystem scanning, broker query, mDNS. Compare latency, reliability, complexity.
4. **Lifecycle state machine** (20 min) — States, transitions, notifications.
5. **Liveness detection** (15 min) — Socket probe, PID check, heartbeat. Trade-offs.

## Technical Constraints

- Must work without a running broker (peer-to-peer discovery fallback)
- Must support macOS and Linux filesystem conventions
- Session names must be filesystem-safe (used as socket filenames)
- Discovery must complete in <100ms for interactive use
- Must handle stale entries (process died but socket file remains)

## Scope Fence

**IN:** Naming, registration, discovery, lifecycle states, liveness detection, stale entry cleanup.
**OUT:** Cross-machine discovery (T-011). Security/auth for discovery (T-008). Message routing (T-005).

## Acceptance Criteria

- [ ] Problem statement validated
- [ ] Assumptions tested
- [ ] Go/No-Go decision made
- [ ] Session lifecycle state machine defined
- [ ] Discovery protocol designed
- [ ] Naming scheme decided
- [ ] Research artifact committed to docs/reports/

## Go/No-Go Criteria

**GO if:**
- Discovery works without mandatory broker infrastructure
- Lifecycle state machine covers crash, graceful shutdown, network partition
- Naming scheme satisfies D3 (usable) without sacrificing D2 (reliable, no collisions)

**NO-GO if:**
- Discovery requires mandatory infrastructure violating D4 (portability)
- No reliable stale-entry cleanup mechanism exists

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

### 2026-03-08T15:18:46Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
