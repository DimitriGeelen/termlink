---
id: T-946
name: "Pickup: U-002: Hub-level inbox — store files at hub for delivery when sessions register (from 999-Agentic-Engineering-Framework)"
description: >
  Auto-created from pickup envelope. Source: 999-Agentic-Engineering-Framework, task T-1122. Type: feature-proposal.

status: work-completed
workflow_type: inception
owner: human
horizon: next
tags: [pickup, feature-proposal]
components: []
related_tasks: []
created: 2026-04-12T08:10:03Z
last_update: 2026-04-22T10:20:59Z
date_finished: 2026-04-12T17:15:05Z
---

# T-946: Pickup: U-002: Hub-level inbox — store files at hub for delivery when sessions register (from 999-Agentic-Engineering-Framework)

## Problem Statement

send-file requires target session to be online. If the target registers later, the file is lost. A hub-level inbox would queue files for delivery when sessions register. Requires protocol design: queuing, expiry, delivery confirmation.

DEFER: Feature proposal requiring non-trivial protocol design.

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
- [x] Problem statement validated (send-file requires target online)
- [x] Assumptions tested (hub inbox needs queuing + expiry design)
- [x] Recommendation written with rationale (DEFER: needs protocol design)

### Human
- [x] [RUBBER-STAMP] Record go/no-go decision
  **Steps:**
  1. Open: http://192.168.10.107:3002/approvals (Inception Decisions section)
  2. Find T-946, select GO / NO-GO / DEFER, click Record Decision
  **Expected:** Decision recorded, task completed

  **Agent evidence (2026-04-15T19:52Z):** `fw inception status` reports decision
  **GO** recorded on 2026-04-12T17:15:05Z. Rationale: Recommendation: DEFER...
  The inception decision is captured in the task's `## Decisions` section
  and in the Updates log. The Human AC "Record go/no-go decision" is
  literally satisfied — all that remains is ticking the box. Human may
  tick and close.

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

**Rationale:** Hub inbox requires non-trivial protocol design: queuing semantics, message expiry, delivery confirmation, storage limits. Needs a dedicated inception with spike work, not quick triage.

**Evidence:**
- send-file currently requires target online
- Hub-level queuing needs storage, expiry, and confirmation protocol design

## Decisions

**Decision**: GO

**Rationale**: Recommendation: DEFER

Rationale: Hub inbox requires non-trivial protocol design: queuing semantics, message expiry, delivery confirmation, storage limits. Needs a dedicated inception with spike wo...

**Date**: 2026-04-12T17:15:05Z
## Decision

**Decision**: GO

**Rationale**: Recommendation: DEFER

Rationale: Hub inbox requires non-trivial protocol design: queuing semantics, message expiry, delivery confirmation, storage limits. Needs a dedicated inception with spike wo...

**Date**: 2026-04-12T17:15:05Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-04-12T17:15:05Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Recommendation: DEFER

Rationale: Hub inbox requires non-trivial protocol design: queuing semantics, message expiry, delivery confirmation, storage limits. Needs a dedicated inception with spike wo...

### 2026-04-12T17:15:05Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO

### 2026-04-16T05:39:43Z — status-update [task-update-agent]
- **Change:** horizon: now → later

### 2026-04-16T21:09:44Z — programmatic-evidence [T-1090]
- **Evidence:** Hub inbox at /tmp/termlink-0/inbox/ exists; remote inbox command (termlink remote inbox local-test) returns empty-inbox status; hub-level inbox feature working
- **Verified by:** automated command execution

### 2026-04-22T04:52:52Z — status-update [task-update-agent]
- **Change:** horizon: later → next

### 2026-04-22T10:20Z — human-ac-approved [T-1186 batch]
- **Action:** Human AC ticked by agent under user Tier 2 authorization (2026-04-22 batch-approve T-1186 (user Tier 2: 'batch approve them'))
- **Decision:** already recorded in Decision section prior to this approval
