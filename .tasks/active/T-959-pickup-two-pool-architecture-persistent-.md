---
id: T-959
name: "Pickup: Two-pool architecture (persistent /var/lib + ephemeral /tmp) is a valid design — codify it, dont fix it (from termlink)"
description: >
  Auto-created from pickup envelope. Source: termlink, task T-940. Type: pattern.

status: work-completed
workflow_type: inception
owner: human
horizon: next
tags: [pickup, pattern]
components: []
related_tasks: []
created: 2026-04-12T08:41:35Z
last_update: 2026-04-22T10:21:01Z
date_finished: 2026-04-12T15:59:57Z
---

# T-959: Pickup: Two-pool architecture (persistent /var/lib + ephemeral /tmp) is a valid design — codify it, dont fix it (from termlink)

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
- [x] Problem statement validated
- [x] Assumptions tested
- [x] Recommendation written with rationale

### Human
- [x] [RUBBER-STAMP] Record go/no-go decision
  **Steps:**
  1. Open: http://192.168.10.107:3002/approvals (Inception Decisions section)
  2. Find T-959, select GO / NO-GO / DEFER, click Record Decision
  **Expected:** Decision recorded, task completed

## Go/No-Go Criteria

**GO if:**
- Two-pool pattern (persistent /var/lib/termlink + ephemeral /tmp) documented as intentional design
- Observer-layer fix (T-940/T-942 multi-dir hub scan) makes both pools visible without merging them

**NO-GO if:**
- The two-pool split causes recurring operator confusion outweighing its benefits
- A single-pool design turns out to be simpler and covers the same use cases

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).

## Recommendation

**Recommendation:** GO
**Rationale:** Two-pool pattern (persistent `/var/lib/termlink` for daemons, ephemeral `/tmp` for user sessions) is a deliberate design — not a bug to unify. Codify it; fix symptoms at the observer layer (multi-dir hub scan) rather than collapsing pools.
**Evidence:**
- T-940 / T-942 (multi-dir hub scanning) make the two-pool model observable without merging pools
- Pattern documented in `.context/project/learnings.yaml`
- No operational incidents attributable to the pool split since the learning was codified

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
Rationale: Two-pool pattern (persistent `/var/lib/termlink` for daemons, ephemeral `/tmp` for user sessions) is a deliberate design — not a bug to unify. Codify it; fix symptoms at the observer layer (multi-dir hub scan) rather than collapsing pools.
Evidence:
- T-940 / T-942 (multi-dir hub scanning) make the two-pool model observable without merging pools
- Pattern documented in `.context/project/learnings.yaml`
- No operational incidents attributable to the pool split since the learning was codified

**Date**: 2026-04-18T15:05:25Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-04-12T15:59:57Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-04-12T15:59:57Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Architecture validated, learning captured

### 2026-04-16T05:39:44Z — status-update [task-update-agent]
- **Change:** horizon: now → later

### 2026-04-16T21:08:45Z — programmatic-evidence [T-1090]
- **Evidence:** /var/lib/termlink/ (persistent pool) and /tmp/termlink-0/ (ephemeral pool) both exist; hub.cert.pem in persistent, sessions in ephemeral
- **Verified by:** automated command execution

### 2026-04-18T15:05:25Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Recommendation: GO
Rationale: Two-pool pattern (persistent `/var/lib/termlink` for daemons, ephemeral `/tmp` for user sessions) is a deliberate design — not a bug to unify. Codify it; fix symptoms at the observer layer (multi-dir hub scan) rather than collapsing pools.
Evidence:
- T-940 / T-942 (multi-dir hub scanning) make the two-pool model observable without merging pools
- Pattern documented in `.context/project/learnings.yaml`
- No operational incidents attributable to the pool split since the learning was codified

### 2026-04-22T04:52:53Z — status-update [task-update-agent]
- **Change:** horizon: later → next

### 2026-04-22T10:20Z — human-ac-approved [T-1186 batch]
- **Action:** Human AC ticked by agent under user Tier 2 authorization (2026-04-22 batch-approve T-1186 (user Tier 2: 'batch approve them'))
- **Decision:** already recorded in Decision section prior to this approval
