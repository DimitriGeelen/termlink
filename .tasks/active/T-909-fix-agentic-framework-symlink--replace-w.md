---
id: T-909
name: "Fix .agentic-framework symlink — replace with vendored copy"
description: >
  Inception: TermLink's .agentic-framework is a symlink to /opt/999-Agentic-Engineering-Framework instead of a vendored copy (other projects use vendored copies). Causes PROJECT_ROOT resolution bugs in tooling like watchtower.sh and potentially shared state pollution in the framework repo. Evaluate risks, design the fix, decide go/no-go.

status: work-completed
workflow_type: inception
owner: human
horizon: now
tags: [infrastructure, symlink, vendor]
components: []
related_tasks: [T-288, T-908]
created: 2026-04-11T10:49:11Z
last_update: 2026-04-12T07:09:10Z
date_finished: 2026-04-11T12:21:19Z
---

# T-909: Fix .agentic-framework symlink — replace with vendored copy

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
- [ ] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `fw task review T-XXX` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

**GO if:**
- [Criterion 1]
- [Criterion 2]

**NO-GO if:**
- [Criterion 1]
- [Criterion 2]

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).

## Recommendation

<!-- REQUIRED before fw inception decide. Write your recommendation here (T-974).
     Watchtower reads this section — if it's empty, the human sees nothing.
     Format:
     **Recommendation:** GO / NO-GO / DEFER
     **Rationale:** Why (cite evidence from exploration)
     **Evidence:**
     - Finding 1
     - Finding 2
-->

## Decisions

**Decision**: GO

**Rationale**: 3-angle risk eval converged on GO-WITH-CAVEATS. Use fw vendor (56MB, exclusions) not cp -r (349MB     
  pollution). Pre-flight kills live contaminating watchtower PID 1471772 directly, stages atomic rollback via .symlink.bak, vendors, restarts watchtower with explicit PROJECT_ROOT.

**Date**: 2026-04-11T12:21:19Z
## Decision

**Decision**: GO

**Rationale**: 3-angle risk eval converged on GO-WITH-CAVEATS. Use fw vendor (56MB, exclusions) not cp -r (349MB     
  pollution). Pre-flight kills live contaminating watchtower PID 1471772 directly, stages atomic rollback via .symlink.bak, vendors, restarts watchtower with explicit PROJECT_ROOT.

**Date**: 2026-04-11T12:21:19Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-04-11T12:21:19Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** 3-angle risk eval converged on GO-WITH-CAVEATS. Use fw vendor (56MB, exclusions) not cp -r (349MB     
  pollution). Pre-flight kills live contaminating watchtower PID 1471772 directly, stages atomic rollback via .symlink.bak, vendors, restarts watchtower with explicit PROJECT_ROOT.

### 2026-04-11T12:21:19Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
