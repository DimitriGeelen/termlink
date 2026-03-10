---
id: T-008
name: "IT-006: Security model — capability-based access"
description: >
  Design auth, consent prompts, command allowlists, Tier 0 integration

status: started-work
workflow_type: inception
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-08T14:19:42Z
last_update: 2026-03-10T20:25:59Z
date_finished: null
---

# T-008: IT-006: Security model — capability-based access

## Problem Statement

Any local process can connect to any TermLink session's Unix socket and execute arbitrary commands (exec, inject, spawn). The reflection fleet security analysis (docs/reports/reflection-result-security.md) identified two critical gaps: (1) no sender authentication — `CommonParams.sender` is self-reported and never validated, (2) command injection in `executor.rs` — user strings passed directly to `sh -c`. T-064 fixed the injection vector; this inception explores the broader auth/authz model needed for multi-agent and multi-user deployment.

## Assumptions

- A1: SO_PEERCRED on Unix sockets can reliably identify connecting processes (Linux; macOS has LOCAL_PEERCRED equivalent)
- A2: Capability-based tokens are more flexible than ACL-based controls for agent-to-agent delegation
- A3: The current single-user model (socket dir 0o700) is sufficient for local development; auth is needed for shared/production use
- A4: Existing Tier 0 framework enforcement can integrate with per-method authorization checks

## Exploration Plan

1. **Spike 1 (1h):** Research SO_PEERCRED/LOCAL_PEERCRED cross-platform availability. Test on macOS and Linux.
2. **Spike 2 (1h):** Prototype capability token format — what fields? Expiry? Scope (per-method, per-session, per-command)?
3. **Spike 3 (1h):** Map all 26 CLI commands to required permission levels. Which are read-only? Which are destructive?
4. **Design (1h):** Draft auth model document. Present options: token-based vs. socket-credential vs. hybrid.

## Technical Constraints

- macOS uses `LOCAL_PEERCRED` (not `SO_PEERCRED`) — different API, same concept
- Unix sockets only; no TCP/TLS auth needed yet (see T-011 for distributed topology)
- Must not break existing single-user workflow — auth should be opt-in or transparent
- `--dangerously-skip-permissions` used in 4 test scripts — needs replacement strategy

## Scope Fence

**IN scope:** Authentication (who is connecting), authorization (what can they do), capability token design, per-method permission mapping.
**OUT of scope:** TLS/mTLS for TCP transport (T-011), encrypted socket communication, user management UI, OAuth/OIDC integration.

## Acceptance Criteria

- [x] Problem statement validated
- [x] Assumptions tested
- [x] Go/No-Go decision made

## Go/No-Go Criteria

**GO if:**
- SO_PEERCRED/LOCAL_PEERCRED works reliably on both macOS and Linux
- A capability token model can be designed that doesn't break existing single-user workflow
- Permission mapping for all 26 commands is clear and enforceable

**NO-GO if:**
- Cross-platform socket credential APIs are too inconsistent for reliable use
- Auth overhead adds >5ms latency per RPC call (unacceptable for interactive use)
- The complexity of capability management exceeds the threat model for local-only deployment

## Verification

<!-- Shell commands that MUST pass before work-completed. One per line.
     Lines starting with # are comments. Empty lines ignored.
     The completion gate runs each command — if any exits non-zero, completion is blocked.
     For inception tasks, verification is often not needed (decisions, not code).
-->

## Decisions

**Decision**: GO

**Rationale**: SO_PEERCRED/LOCAL_PEERCRED works cross-platform (<0.1ms), 4-tier permission model maps cleanly to 17 RPC methods, Phase 1 (UID check) preserves single-user UX with zero config

**Date**: 2026-03-10T20:36:01Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-03-10T08:45:03Z — status-update [task-update-agent]
- **Change:** horizon: later → now

### 2026-03-10T20:25:59Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-10T20:36:01Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** SO_PEERCRED/LOCAL_PEERCRED works cross-platform (<0.1ms), 4-tier permission model maps cleanly to 17 RPC methods, Phase 1 (UID check) preserves single-user UX with zero config
