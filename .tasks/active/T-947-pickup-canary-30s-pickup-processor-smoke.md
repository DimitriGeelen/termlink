---
id: T-947
name: "Pickup: Canary: 30s pickup processor smoke test — safe to discard (from termlink)"
description: >
  Auto-created from pickup envelope. Source: termlink, task T-940. Type: learning.

status: work-completed
workflow_type: inception
owner: human
horizon: next
tags: [pickup, learning]
components: []
related_tasks: []
created: 2026-04-12T08:10:05Z
last_update: 2026-04-22T10:20:59Z
date_finished: 2026-04-12T13:03:33Z
---

# T-947: Pickup: Canary: 30s pickup processor smoke test — safe to discard (from termlink)

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
- [x] [RUBBER-STAMP] Record go/no-go decision
  **Steps:**
  1. Open: http://192.168.10.107:3002/approvals (Inception Decisions section)
  2. Find T-947, select GO / NO-GO / DEFER, click Record Decision
  **Expected:** Decision recorded, task completed

## Go/No-Go Criteria

**GO if:**
- Pickup processor demonstrably consumes envelopes (cron firing, no backlog, no errors)
- Canary learning absorbed — no build work required by this task

**NO-GO if:**
- Pickup processor silently drops envelopes or accumulates backlog
- Canary reveals a defect in the pickup pipeline that needs remediation

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).

## Recommendation

**Recommendation:** GO
**Rationale:** Canary by design — the test was explicitly "safe to discard" and its purpose was to verify the pickup processor wakes up and consumes envelopes. No build work required; the learning ("canary tests are valid lightweight smoke tests") is absorbed.
**Evidence:**
- Pickup cron `/etc/cron.d/agentic-pickup-termlink` active at 1-min interval (programmatic evidence from T-1090, 2026-04-16)
- Zero errors in syslog over observation window
- Processor cadence tuned to 1-min for headroom (see T-950)

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
Rationale: Canary by design — the test was explicitly "safe to discard" and its purpose was to verify the pickup processor wakes up and consumes envelopes. No build work required; the learning ("canary tests are valid lightweight smoke tests") is absorbed.
Evidence:
- Pickup cron `/etc/cron.d/agentic-pickup-termlink` active at 1-min interval (programmatic evidence from T-1090, 2026-04-16)
- Zero errors in syslog over observation window
- Processor cadence tuned to 1-min for headroom (see T-950)

**Date**: 2026-04-18T14:56:48Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-04-12T13:03:33Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-04-12T13:03:33Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Canary test message — explicitly safe to discard

### 2026-04-16T05:39:43Z — status-update [task-update-agent]
- **Change:** horizon: now → later

### 2026-04-16T21:05:40Z — programmatic-evidence [T-1090]
- **Evidence:** Pickup cron running at 1-min interval (/etc/cron.d/agentic-pickup-termlink confirms * * * * *)
- **Verified by:** automated command execution

### 2026-04-18T14:56:48Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Recommendation: GO
Rationale: Canary by design — the test was explicitly "safe to discard" and its purpose was to verify the pickup processor wakes up and consumes envelopes. No build work required; the learning ("canary tests are valid lightweight smoke tests") is absorbed.
Evidence:
- Pickup cron `/etc/cron.d/agentic-pickup-termlink` active at 1-min interval (programmatic evidence from T-1090, 2026-04-16)
- Zero errors in syslog over observation window
- Processor cadence tuned to 1-min for headroom (see T-950)

### 2026-04-22T04:52:52Z — status-update [task-update-agent]
- **Change:** horizon: later → next

### 2026-04-22T10:20Z — human-ac-approved [T-1186 batch]
- **Action:** Human AC ticked by agent under user Tier 2 authorization (2026-04-22 batch-approve T-1186 (user Tier 2: 'batch approve them'))
- **Decision:** already recorded in Decision section prior to this approval
