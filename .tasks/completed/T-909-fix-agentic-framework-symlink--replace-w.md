---
id: T-909
name: "Fix .agentic-framework symlink — replace with vendored copy"
description: >
  Inception: TermLink's .agentic-framework is a symlink to /opt/999-Agentic-Engineering-Framework instead of a vendored copy (other projects use vendored copies). Causes PROJECT_ROOT resolution bugs in tooling like watchtower.sh and potentially shared state pollution in the framework repo. Evaluate risks, design the fix, decide go/no-go.

status: work-completed
workflow_type: inception
owner: human
horizon: next
tags: [infrastructure, symlink, vendor]
components: []
related_tasks: [T-288, T-908]
created: 2026-04-11T10:49:11Z
last_update: 2026-04-23T19:30:27Z
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
- [x] Problem statement validated
- [x] Assumptions tested
- [x] Recommendation written with rationale

### Human
- [x] [RUBBER-STAMP] Record go/no-go decision
  **Steps:**
  1. Open: http://192.168.10.107:3002/approvals (Inception Decisions section)
  2. Find T-909, select GO / NO-GO / DEFER, click Record Decision
  **Expected:** Decision recorded, task completed

  **Agent evidence (2026-04-15T19:52Z):** `fw inception status` reports decision
  **GO** recorded on 2026-04-11T12:21:19Z. Rationale: 3-angle risk eval converged on GO-WITH-CAVEATS. Use fw vendor (56MB, exclusions) not cp -r (349MB     
  pollution). Pre-flight kills live contaminating watchtower PID 1471772 directly, stages atomic ...
  The inception decision is captured in the task's `## Decisions` section
  and in the Updates log. The Human AC "Record go/no-go decision" is
  literally satisfied — all that remains is ticking the box. Human may
  tick and close.

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

### 2026-04-16T05:40:15Z — status-update [task-update-agent]
- **Change:** horizon: now → later

### 2026-04-16T21:04:50Z — programmatic-evidence [T-1090]
- **Evidence:** .agentic-framework is a real directory (drwxr-xr-x), not a symlink — vendored copy confirmed
- **Verified by:** automated command execution

### 2026-04-22T04:52:51Z — status-update [task-update-agent]
- **Change:** horizon: later → next

### 2026-04-22T10:20Z — human-ac-approved [T-1186 batch]
- **Action:** Human AC ticked by agent under user Tier 2 authorization (2026-04-22 batch-approve T-1186 (user Tier 2: 'batch approve them'))
- **Decision:** already recorded in Decision section prior to this approval
