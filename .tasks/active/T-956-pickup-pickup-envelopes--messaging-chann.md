---
id: T-956
name: "Pickup: Pickup envelopes = messaging channel, termlink sessions = execution channel — agents must know the distinction (from termlink)"
description: >
  Auto-created from pickup envelope. Source: termlink, task T-940. Type: learning.

status: work-completed
workflow_type: inception
owner: human
horizon: later
tags: [pickup, learning]
components: []
related_tasks: []
created: 2026-04-12T08:40:35Z
last_update: 2026-04-18T15:05:20Z
date_finished: 2026-04-12T15:59:20Z
---

# T-956: Pickup: Pickup envelopes = messaging channel, termlink sessions = execution channel — agents must know the distinction (from termlink)

## Problem Statement

<!-- What problem are we exploring? For whom? Why now? -->

## Assumptions

<!-- Key assumptions to test. Register with: fw assumption add "Statement" --task T-XXX -->

## Exploration Plan

<!-- How will we validate assumptions? Spikes, prototypes, research? Time-box each. -->

## Technical Constraints

<!-- What platform, browser, network, or hardware constraints apply?
     For web apps: HTTPS requirements, browser API restrictions, CORS, device support.
     For hardware APIs (mic, camera, GPS, Bluetooth): access requirements, permissions model.
     For infrastructure: network topology, firewall rules, latency bounds.
     Fill this BEFORE building. Discovering constraints after implementation wastes sessions. -->

## Scope Fence

<!-- What's IN scope for this exploration? What's explicitly OUT? -->

## Acceptance Criteria

### Agent
- [ ] Problem statement validated
- [ ] Assumptions tested
- [ ] Recommendation written with rationale

### Human
- [ ] [RUBBER-STAMP] Record go/no-go decision
  **Steps:**
  1. Open: http://192.168.10.107:3002/approvals (Inception Decisions section)
  2. Find T-956, select GO / NO-GO / DEFER, click Record Decision
  **Expected:** Decision recorded, task completed

## Go/No-Go Criteria

**GO if:**
- CLAUDE.md documents the pickup-envelope (messaging) vs termlink-session (execution) distinction
- G-020 gate blocks pickup messages mis-scoped as build instructions (already shipping)

**NO-GO if:**
- Agents repeatedly treat pickup envelopes as execution requests despite docs + gate
- The distinction proves impossible to codify without a larger redesign

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).

## Recommendation

**Recommendation:** GO
**Rationale:** Distinction is a mental model, not code. Already codified in CLAUDE.md's `Pickup Message Handling` section (G-020 gate), which blocks edits when pickup messages are mis-scoped as build instructions. No standalone build work needed — operator/agent guidance is structurally enforced.
**Evidence:**
- CLAUDE.md `Pickup Message Handling (G-020, T-469)` section present
- G-020 gate blocks edits on placeholder ACs produced by mis-scoped pickups
- Learning stored in `.context/project/learnings.yaml`

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

**Decision**: GO

**Rationale**: Recommendation: GO
Rationale: Distinction is a mental model, not code. Already codified in CLAUDE.md's `Pickup Message Handling` section (G-020 gate), which blocks edits when pickup messages are mis-scoped as build instructions. No standalone build work needed — operator/agent guidance is structurally enforced.
Evidence:
- CLAUDE.md `Pickup Message Handling (G-020, T-469)` section present
- G-020 gate blocks edits on placeholder ACs produced by mis-scoped pickups
- Learning stored in `.context/project/learnings.yaml`

**Date**: 2026-04-18T15:05:20Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-04-12T15:59:20Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-04-12T15:59:20Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Learning captured, no build work needed

### 2026-04-16T05:39:43Z — status-update [task-update-agent]
- **Change:** horizon: now → later

### 2026-04-16T21:08:45Z — programmatic-evidence [T-1090]
- **Evidence:** Pickup cron active at 1-min interval; termlink file send used for cross-agent envelope delivery (T-1079 propagation confirmed)
- **Verified by:** automated command execution

### 2026-04-18T15:05:20Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Recommendation: GO
Rationale: Distinction is a mental model, not code. Already codified in CLAUDE.md's `Pickup Message Handling` section (G-020 gate), which blocks edits when pickup messages are mis-scoped as build instructions. No standalone build work needed — operator/agent guidance is structurally enforced.
Evidence:
- CLAUDE.md `Pickup Message Handling (G-020, T-469)` section present
- G-020 gate blocks edits on placeholder ACs produced by mis-scoped pickups
- Learning stored in `.context/project/learnings.yaml`
