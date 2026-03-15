---
id: T-142
name: "TermLink integration into engineering framework"
description: >
  Inception: TermLink integration into engineering framework

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-15T20:43:40Z
last_update: 2026-03-15T20:55:29Z
date_finished: null
---

# T-142: TermLink integration into engineering framework

## Problem Statement

Framework agent operates in a single terminal. Cannot observe, control, or
coordinate across multiple terminals. TermLink provides all the primitives
(30+ CLI commands, JSON output, event system) to unlock: self-testing,
parallel dispatch, remote control, cross-machine coordination.

## Assumptions

- A1: VALIDATED — TermLink CLI commands are scriptable (--json, exit codes)
- A2: VALIDATED — interact --json captures output + exit code in one call
- A3: VALIDATED — Event system supports coordination (emit/wait/poll/broadcast)
- A4: VALIDATED — Framework agent pattern is simple (AGENT.md + .sh + routing)
- A5: VALIDATED — fw doctor supports optional tool checks (WARN pattern)

## Exploration Plan

1. [x] Enumerate capabilities (11 identified)
2. [x] Human dialogue: prioritization + missing capabilities
3. [x] Research: CLI primitives, agent patterns, fw doctor (3 parallel agents)
4. [x] Define Phase 0 integration
5. [x] Go/No-Go → GO

## Scope Fence

**IN:** Defining what the framework needs, phased rollout, ownership boundary
**OUT:** Building the framework integration (that's framework project work)

## Acceptance Criteria

- [x] Problem statement validated
- [x] Assumptions tested (A1-A5)
- [x] Research artifact written (docs/reports/T-142-framework-termlink-integration.md)
- [x] Go/No-Go decision made → GO

## Verification

test -f docs/reports/T-142-framework-termlink-integration.md

## Decisions

**Decision**: GO

**Rationale**: All primitives exist in TermLink today. 30+ CLI commands with JSON output, reliable exit codes. interact --json is the star primitive. Framework integration is bounded: fw doctor check + agents/termlink/ wrapper + fw route. No TermLink changes needed for Phase 0. Phased rollout from self-test (already validated) through parallel dispatch to remote control and cross-machine.

**Date**: 2026-03-15T21:05:53Z
## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-03-15T21:05:53Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** All primitives exist in TermLink today. 30+ CLI commands with JSON output, reliable exit codes. interact --json is the star primitive. Framework integration is bounded: fw doctor check + agents/termlink/ wrapper + fw route. No TermLink changes needed for Phase 0. Phased rollout from self-test (already validated) through parallel dispatch to remote control and cross-machine.
