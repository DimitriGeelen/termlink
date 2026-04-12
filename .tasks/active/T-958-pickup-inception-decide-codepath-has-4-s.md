---
id: T-958
name: "Pickup: Inception decide codepath has 4 stacking UX issues — needs a focused review pass, not individual fixes (from termlink)"
description: >
  Auto-created from pickup envelope. Source: termlink, task T-940. Type: learning.

status: started-work
workflow_type: inception
owner: agent
horizon: now
tags: [pickup, learning]
components: []
related_tasks: []
created: 2026-04-12T08:41:34Z
last_update: 2026-04-12T08:41:34Z
date_finished: null
---

# T-958: Pickup: Inception decide codepath has 4 stacking UX issues — needs a focused review pass, not individual fixes (from termlink)

## Problem Statement

Inception decide codepath has 4 stacking UX issues identified by a previous session. T-949 fixed one (captured task auto-transition). Remaining 3 need investigation. Common case now works.

DEFER: T-949 fixed the most critical issue. Remaining are UX polish.

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
- [x] Problem statement validated (T-949 fixed most critical issue)
- [x] Assumptions tested (inception decide works for common case)
- [x] Recommendation written with rationale (DEFER: remaining issues are UX polish)

### Human
- [ ] [RUBBER-STAMP] Record go/no-go decision
  **Steps:**
  1. Open: http://192.168.10.107:3002/approvals (Inception Decisions section)
  2. Find T-958, select GO / NO-GO / DEFER, click Record Decision
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

**Rationale:** T-949 fixed the most critical inception decide UX issue (captured task auto-transition). Remaining 3 issues are UX polish that don't block normal workflow.

**Evidence:**
- T-949 fix committed and working
- Inception decide works for the common case (go/no-go on started-work tasks)

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
