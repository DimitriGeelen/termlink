---
id: T-232
name: "Terminal shadow sessions — read-only monitoring of agent terminals via hub"
description: >
  Inception: Terminal shadow sessions — read-only monitoring of agent terminals via hub

status: work-completed
workflow_type: inception
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-23T08:58:57Z
last_update: 2026-03-23T09:09:23Z
date_finished: 2026-03-23T09:09:23Z
---

# T-232: Terminal shadow sessions — read-only monitoring of agent terminals via hub

## Problem Statement

When multiple agents run through a TermLink hub, the human operator cannot see what's happening inside agent terminals in real-time. Existing tools are either one-shot snapshots (`output`) or exclusive interactive sessions (`attach`/`stream`). There's no read-only, non-intrusive "shadow" mode that lets you monitor one or more agent terminals without affecting them. For whom: human operators supervising multi-agent workflows. Why now: agent fleet usage is growing, blind supervision is antifragility gap.

## Assumptions

- A-1: Users want to monitor agent terminals without interrupting the agent's work
- A-2: The existing data plane broadcast channel can support multiple read-only subscribers
- A-3: Read-only monitoring at Observe permission scope is sufficient (no need for new scope tier)
- A-4: Single-session shadowing is the MVP; multi-session dashboard is stretch

## Exploration Plan

1. **Spike 1**: Can `broadcast::channel` support N read-only subscribers alongside one interactive client? (code review, 15min)
2. **Spike 2**: Design read-only data plane handshake — how does a shadow client connect and get rejected for Input frames? (design, 30min)
3. **Dialogue**: Review approach with human, validate scope and UX expectations

## Technical Constraints

- Data plane uses binary framing over Unix sockets (local) or TCP (remote)
- `tokio::broadcast::channel` already multicasts — adding subscribers is architecturally native
- Permission model must distinguish interactive (Control/Execute) from observe-only
- Terminal output is raw bytes (PTY output), not structured — shadow client needs a terminal emulator to render meaningfully

## Scope Fence

**IN:** Read-only mirror of a single session's terminal output via data plane
**IN:** CLI command `termlink mirror <session>` (local + remote)
**IN:** Raw output mode (MVP), rendered mode as research + later task
**OUT:** Multi-session mirror with TUI (later task)
**OUT:** Recording/replay (different feature)

## Acceptance Criteria

- [x] Problem statement validated with human
- [x] Assumptions tested (A-2 confirmed: broadcast channel natively supports multi-subscriber)
- [x] Architecture approach decided (data plane mirror mode, skip write loop)
- [x] Go/No-Go decision made (GO)

## Go/No-Go Criteria

**GO if:**
- Broadcast channel natively supports multiple subscribers (no major refactor)
- Clear UX for `termlink shadow` that's distinct from `termlink attach`
- Estimated build effort < 400 lines of Rust

**NO-GO if:**
- Data plane architecture requires fundamental redesign for multi-client
- Permission model needs breaking changes
- Existing `termlink output --follow` (polling) is "good enough"

## Verification

<!-- Shell commands that MUST pass before work-completed. One per line.
     Lines starting with # are comments. Empty lines ignored.
     The completion gate runs each command — if any exits non-zero, completion is blocked.
     For inception tasks, verification is often not needed (decisions, not code).
-->

## Decisions

**Decision**: GO

**Rationale**: Broadcast channel natively supports multi-subscriber. ~400 lines, low-medium complexity. Name: mirror. Scope: single session, raw+rendered (selectable), local+remote.

**Date**: 2026-03-23T09:09:23Z
## Decision

**Decision**: GO

**Rationale**: Broadcast channel natively supports multi-subscriber. ~400 lines, low-medium complexity. Name: mirror. Scope: single session, raw+rendered (selectable), local+remote.

**Date**: 2026-03-23T09:09:23Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-03-23T09:08:38Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Broadcast channel natively supports multi-subscriber (no refactor). ~400 lines, low-medium complexity. Clear UX: mirror=read-only vs attach=interactive. Local+remote.

### 2026-03-23T09:08:47Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Broadcast channel natively supports multi-subscriber (no refactor). ~400 lines, low-medium complexity. Clear UX: mirror=read-only vs attach=interactive. Local+remote.

### 2026-03-23T09:08:59Z — status-update [task-update-agent]
- **Change:** owner: human → agent

### 2026-03-23T09:08:59Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Broadcast channel natively supports multi-subscriber (no refactor). ~400 lines, low-medium complexity. Clear UX: mirror=read-only vs attach=interactive. Local+remote.

### 2026-03-23T09:09:23Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Broadcast channel natively supports multi-subscriber. ~400 lines, low-medium complexity. Name: mirror. Scope: single session, raw+rendered (selectable), local+remote.

### 2026-03-23T09:09:23Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
