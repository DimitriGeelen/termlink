---
id: T-951
name: "Pickup: U-003: send-file reports ok on hub acceptance, not delivery — silent file loss to event-only sessions (from 999-Agentic-Engineering-Framework)"
description: >
  Auto-created from pickup envelope. Source: 999-Agentic-Engineering-Framework, task T-1125. Type: bug-report.

status: started-work
workflow_type: inception
owner: agent
horizon: now
tags: [pickup, bug-report]
components: []
related_tasks: []
created: 2026-04-12T08:21:31Z
last_update: 2026-04-12T08:21:31Z
date_finished: null
---

# T-951: Pickup: U-003: send-file reports ok on hub acceptance, not delivery — silent file loss to event-only sessions (from 999-Agentic-Engineering-Framework)

## Problem Statement

send-file returns ok:true when hub accepts the file, not when recipient receives it. Event-only sessions never receive files. This causes silent file loss. Fix requires delivery confirmation protocol.

DEFER: Subsumed by T-946 (hub inbox addresses root cause).

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
- [x] Problem statement validated (ok:true = hub accepted, not delivered)
- [x] Assumptions tested (T-946 hub inbox addresses root cause)
- [x] Recommendation written with rationale (DEFER: subsumed by T-946)

### Human
- [ ] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `fw task review T-XXX` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

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

**Rationale:** Subsumed by T-946. Hub inbox addresses the root cause (files for offline sessions) rather than the symptom (misleading ok:true response).

**Evidence:**
- ok:true means hub accepted, not recipient received
- T-946 hub inbox would provide actual delivery confirmation

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
