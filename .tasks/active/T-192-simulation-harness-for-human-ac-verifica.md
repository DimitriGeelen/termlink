---
id: T-192
name: "Simulation harness for human AC verification"
description: >
  Explore whether human ACs can be verified via automated simulation using
  TermLink spawn/inject/output instead of manual interactive testing.
  11 reopened tasks need proper verification.

status: captured
workflow_type: inception
owner: human
horizon: later
tags: [verification, simulation, testing]
components: []
related_tasks: [T-191, T-124, T-126, T-127, T-148, T-156, T-157, T-158, T-160, T-178, T-188]
created: 2026-03-20T06:05:00Z
last_update: 2026-04-16T05:38:15Z
date_finished: null
---

# T-192: Simulation harness for human AC verification

## Problem Statement

11 tasks were rubber-stamped as "structural pass" because their human ACs appeared
to require interactive sessions. We can use TermLink's own spawn/inject/output to
simulate the human steps. Research artifact: `docs/reports/T-192-simulation-harness-design.md`

## Assumptions

- A1: dispatch.sh works with non-Claude commands (echo, sleep) as substitutes
- A2: tl-claude.sh lifecycle can be tested without a real Claude API call
- A3: TermLink pty inject Enter works against any interactive program (not just Claude TUI)
- A4: A framework session can be spawned in an ephemeral test project
- A5: Document "clarity" can be proxied by structural completeness checks

## Exploration Plan

5 spikes detailed in research artifact. Total budget: ~55 min.
- Spike 1: Dispatch simulation (T-124/126/127) — 15 min
- Spike 2: tl-claude lifecycle (T-156/158) — 10 min
- Spike 3: PTY inject Enter (T-178) — 5 min
- Spike 4: Framework pickup simulation (T-148/157/160) — 20 min (API cost)
- Spike 5: Document structure check (T-188/191) — 5 min

## Technical Constraints

- tmux must be available (macOS default: yes)
- TermLink binary must be built and on PATH
- Spike 4 requires Claude API tokens (real invocation)
- Framework must be installed (`fw` available)

## Scope Fence

**IN:** Simulation scripts for 11 specific tasks, reusable patterns, /self-test integration
**OUT:** General test framework, CI/CD integration, TermLink mocking

## Acceptance Criteria

### Agent
- [x] Research artifact created with 5 spike plans
- [x] Spikes 1-3 executed (free, no API cost)
- [x] Spike 4 feasibility assessed (framework session spawn)
- [x] Spike 5 executed (document structure checks)
- [x] Go/No-Go decision made with evidence

### Human
- [x] [REVIEW] Review spike results and approve direction

## Go/No-Go Criteria

**GO if:**
- Spikes 1-3 pass (dispatch, tl-claude, Enter inject) — free to run
- Spike 4 feasibility confirmed (framework session spawnable)
- Total simulation time < 5 min per run

**NO-GO if:**
- TermLink spawn/inject too fragile for automation
- Framework session requires manual interaction we can't automate
- API token cost for Spike 4 is prohibitive

## Verification

<!-- Shell commands that MUST pass before work-completed. One per line.
     Lines starting with # are comments. Empty lines ignored.
     The completion gate runs each command — if any exits non-zero, completion is blocked.
     For inception tasks, verification is often not needed (decisions, not code).
-->

## Decisions

**Decision**: GO

**Rationale**: Spikes 1-3 pass: dispatch isolation, tl-claude lifecycle, Enter inject all verified programmatically. 9 of 11 human ACs automatable. Build repeatable simulation script.

**Date**: 2026-03-20T07:47:12Z
## Recommendation

_Backfilled 2026-04-19 under T-1139/T-1112 scope — inception decide ran before `## Recommendation` became a required section. Content mirrors the `## Decision` block below for audit compliance (CTL-027)._

**Decision (retro-captured from Decision block):** GO

**Rationale:** Spikes 1-3 pass: dispatch isolation, tl-claude lifecycle, Enter inject all verified programmatically. 9 of 11 human ACs automatable. Build repeatable simulation script.

## Decision

**Decision**: GO

**Rationale**: Spikes 1-3 pass: dispatch isolation, tl-claude lifecycle, Enter inject all verified programmatically. 9 of 11 human ACs automatable. Build repeatable simulation script.

**Date**: 2026-03-20T07:47:12Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-03-20T07:47:12Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Spikes 1-3 pass: dispatch isolation, tl-claude lifecycle, Enter inject all verified programmatically. 9 of 11 human ACs automatable. Build repeatable simulation script.

### 2026-04-16T05:38:15Z — status-update [task-update-agent]
- **Change:** horizon: now → later
- **Change:** status: started-work → captured (auto-sync)
