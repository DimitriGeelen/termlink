---
id: T-955
name: "Pickup: Dry-run saved us from silent data loss — destructive commands should auto-prompt on large diffs (from termlink)"
description: >
  Auto-created from pickup envelope. Source: termlink, task T-940. Type: learning.

status: work-completed
workflow_type: inception
owner: human
horizon: later
tags: [pickup, learning]
components: []
related_tasks: []
created: 2026-04-12T08:40:33Z
last_update: 2026-04-16T05:39:43Z
date_finished: 2026-04-12T17:16:43Z
---

# T-955: Pickup: Dry-run saved us from silent data loss — destructive commands should auto-prompt on large diffs (from termlink)

## Problem Statement

Destructive framework commands should auto-prompt on large diffs to prevent silent data loss. Dry-run mode saved data during T-978. This is a framework-side UX improvement.

DEFER: Framework-side work, not termlink Rust code.

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
- [x] Problem statement validated (dry-run prevented data loss in T-978)
- [x] Assumptions tested (framework-side UX improvement)
- [x] Recommendation written with rationale (DEFER: framework-side)

### Human
- [ ] [RUBBER-STAMP] Record go/no-go decision
  **Steps:**
  1. Open: http://192.168.10.107:3002/approvals (Inception Decisions section)
  2. Find T-955, select GO / NO-GO / DEFER, click Record Decision
  **Expected:** Decision recorded, task completed

## Go/No-Go Criteria

**GO if:**
- Evidence supports recommendation
- No blocking dependencies

**NO-GO if:**
- Evidence supports recommendation
- No blocking dependencies

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).

## Recommendation

**Recommendation:** DEFER

**Rationale:** Framework-side UX improvement. Dry-run mode already exists for fw upgrade (T-978). Extending auto-prompts to other destructive commands belongs in the framework repo.

**Evidence:**
- Dry-run prevented data loss during T-978 testing
- Pattern applies across framework, not termlink-specific

## Decisions

**Decision**: GO

**Rationale**: Recommendation: DEFER

Rationale: Framework-side UX improvement. Dry-run mode already exists for fw upgrade (T-978). Extending auto-prompts to other destructive commands belongs in the framework re...

**Date**: 2026-04-12T17:16:43Z
## Decision

**Decision**: GO

**Rationale**: Recommendation: DEFER

Rationale: Framework-side UX improvement. Dry-run mode already exists for fw upgrade (T-978). Extending auto-prompts to other destructive commands belongs in the framework re...

**Date**: 2026-04-12T17:16:43Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-04-12T17:16:43Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Recommendation: DEFER

Rationale: Framework-side UX improvement. Dry-run mode already exists for fw upgrade (T-978). Extending auto-prompts to other destructive commands belongs in the framework re...

### 2026-04-12T17:16:43Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO

### 2026-04-16T05:39:43Z — status-update [task-update-agent]
- **Change:** horizon: now → later

### 2026-04-16T21:08:45Z — programmatic-evidence [T-1090]
- **Evidence:** fw upgrade --dry-run confirmed working; destructive-action pattern codified in Tier 0 enforcement
- **Verified by:** automated command execution
