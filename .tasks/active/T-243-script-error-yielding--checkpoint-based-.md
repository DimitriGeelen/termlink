---
id: T-243
name: "Script error yielding — checkpoint-based execution via TermLink sessions"
description: >
  Inception: How does a deterministic script yield mid-execution errors to a stochastic agent WITHOUT crashing? Options: checkpoint-based execution, error stream piping, TermLink session bridge. Open design problem from T-233.

status: captured
workflow_type: inception
owner: human
horizon: next
tags: [T-233, orchestration, error-yielding]
components: []
related_tasks: [T-233]
created: 2026-03-23T13:28:06Z
last_update: 2026-04-22T04:52:50Z
date_finished: null
---

# T-243: Script error yielding — checkpoint-based execution via TermLink sessions

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

- [ ] Problem statement validated
- [ ] Assumptions tested
- [ ] Go/No-Go decision made

## Go/No-Go Criteria

**GO if:**
- [Criterion 1]
- [Criterion 2]

**NO-GO if:**
- [Criterion 1]
- [Criterion 2]

## Verification

<!-- Shell commands that MUST pass before work-completed. One per line.
     Lines starting with # are comments. Empty lines ignored.
     The completion gate runs each command — if any exits non-zero, completion is blocked.
     For inception tasks, verification is often not needed (decisions, not code).
-->

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Recommendation

_Backfilled 2026-04-19 under T-1139/T-1112 scope — inception decide ran before `## Recommendation` became a required section. Content mirrors the `## Decision` block below for audit compliance (CTL-027)._

<!-- Filled at completion via: fw inception decide T-XXX go|no-go --rationale "..." -->

## Decision

<!-- Filled at completion via: fw inception decide T-XXX go|no-go --rationale "..." -->

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-04-19T12:30Z — housekeeping [agent]
- **Action:** T-1139 audit remediation touch. Task remains captured/horizon=later pending operator prioritization; no scope change.
- **Status:** Still backlog — inception not yet entered. Will move when another exploration slot opens.

### 2026-04-22T04:52:50Z — status-update [task-update-agent]
- **Change:** horizon: later → next
