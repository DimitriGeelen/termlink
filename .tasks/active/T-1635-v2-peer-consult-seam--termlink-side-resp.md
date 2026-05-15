---
id: T-1635
name: "v2 peer-consult seam — TermLink-side response to AEF T-1804"
description: >
  TermLink-side inception mirroring AEF T-1804. AEF has filed a cross-repo
  proposal (PROP-T-1804) asking TermLink to agree the transport-vs-semantics
  seam and choose a wakeup option (i/ii/iii) before either repo ships v2
  peer-consult code. This inception produces the TermLink-side response
  artifact (docs/reports/v2-peer-consult-seam-response.md) with answers to
  the four decision points and a bounded cost estimate.

status: work-completed
workflow_type: inception
owner: human
horizon: now
tags: [inception, cross-repo, arc:peer-consult]
components: []
related_tasks: []
created: 2026-05-13T00:00:00Z
last_update: 2026-05-15T19:36:19Z
date_finished: 2026-05-15T19:36:19Z
---

# T-1635: v2 peer-consult seam — TermLink-side response to AEF T-1804

## Problem Statement

AEF T-1804 (completed 2026-05-13) proposes a seam for v2 peer-consult: TermLink
owns transport (channels, events, inbox, delivery, cross-machine relay, wakeup
signal), AEF owns semantics (when to consult, task-context anchoring, audit,
spawn policy). AEF prefers wakeup option (iii) — TermLink emits a generic
"message-arrived-no-consumer" event, AEF subscribes and spawns responders.

TermLink needs to:
1. Confirm or negotiate the seam
2. Choose wakeup option (i/ii/iii) from the TermLink-side perspective
3. Estimate bounded implementation cost
4. Answer the four decision points in PROP-T-1804

## Assumptions

- TermLink's event system already has EventBus broadcast + `event.subscribe`
  long-poll RPC (T-690 shipped), making push subscription available at the
  Rust layer even if CLI/MCP wrappers are not yet wired to it.
- An `inbox.queued` event class (hub emits when message lands in inbox with no
  live consumer) is the right primitive — not a $WAKEUP_CMD hook that would
  force TermLink to store and execute consumer-specific spawn strings.
- This integrates cleanly with T-243 (dialog.* primitive, GO 2026-04-26).

## Exploration Plan

Research-only — read AEF proposal + existing TermLink primitives. No spikes.

## Scope Fence

**IN scope:**
- Response artifact (docs/reports/v2-peer-consult-seam-response.md)
- Answers to PROP-T-1804's four decision points
- Bounded cost estimate for chosen option

**OUT of scope:** Implementation. Build tasks come after joint agreement.

## Acceptance Criteria

### Agent
- [x] Problem statement validated (AEF proposal read, T-690/T-243/T-163 context gathered)
- [x] Assumptions tested (event system, inbox architecture, cross-machine routing confirmed)
- [x] Recommendation written with rationale

### Human
- [ ] [REVIEW] Review response artifact and approve (or amend) before AEF coordination completes
  **Steps:**
  1. `cat /opt/termlink/docs/reports/v2-peer-consult-seam-response.md`
  2. Compare against AEF proposal: `/opt/999-Agentic-Engineering-Framework/docs/proposals/T-1804-cross-agent-conversation-substrate.md`
  3. If amendments needed, update the response file and re-commit
  **Expected:** Seam agreed, wakeup option chosen, bounded cost confirmed
  **If not:** Amend response and notify AEF agent

## Go/No-Go Criteria

**GO if:**
- Seam matches TermLink's existing ownership model
- Chosen wakeup option has bounded cost (≤1 new event class, no new daemon)
- Cross-machine semantics are clean without cross-hub event relay

**NO-GO if:**
- AEF proposal requires TermLink to store/execute consumer-specific spawn commands
- Cross-machine wakeup requires TermLink to add inter-hub event relay (unbounded scope)

## Verification

# Inception — verification not applicable (coordination artifact, not code)

## Recommendation

**Recommendation:** GO (with option-i refinement)

**Rationale:** The seam is correct and maps to existing TermLink ownership.
Wakeup via `inbox.queued` event class (not $WAKEUP_CMD hook) is the cleanest
primitive: generic, machine-local, cross-machine correct via per-hub emission,
cost-bounded at ≤1 event class + ≤15 lines in inbox delivery path.

**Evidence:** See docs/reports/v2-peer-consult-seam-response.md.

## Decision

**Decision**: GO

**Rationale**: Recommendation: GO (with option-i refinement)

Rationale: The seam is correct and maps to existing TermLink ownership.
Wakeup via `inbox.queued` event class (not $WAKEUP_CMD hook) is the cleanest
primitive: generic, machine-local, cross-machine correct via per-hub emission,
cost-bounded at ≤1 event class + ≤15 lines in inbox delivery path.

Evidence: See docs/reports/v2-peer-consult-seam-response.md.

**Date**: 2026-05-15T19:36:19Z

## Updates

### 2026-05-15T19:36:19Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Recommendation: GO (with option-i refinement)

Rationale: The seam is correct and maps to existing TermLink ownership.
Wakeup via `inbox.queued` event class (not $WAKEUP_CMD hook) is the cleanest
primitive: generic, machine-local, cross-machine correct via per-hub emission,
cost-bounded at ≤1 event class + ≤15 lines in inbox delivery path.

Evidence: See docs/reports/v2-peer-consult-seam-response.md.

### 2026-05-15T19:36:19Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
