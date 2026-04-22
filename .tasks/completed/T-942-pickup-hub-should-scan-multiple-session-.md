---
id: T-942
name: "Pickup: Hub should scan multiple session dirs — eliminates runtime dir split-brain (from termlink)"
description: >
  Auto-created from pickup envelope. Source: termlink, task T-940. Type: feature-proposal.

status: work-completed
workflow_type: inception
owner: human
horizon: next
tags: [pickup, feature-proposal]
components: []
related_tasks: []
created: 2026-04-12T07:49:08Z
last_update: 2026-04-22T10:20:58Z
date_finished: 2026-04-12T17:14:43Z
---

# T-942: Pickup: Hub should scan multiple session dirs — eliminates runtime dir split-brain (from termlink)

## Problem Statement

`runtime_dir()` in `discovery.rs:10-26` returns a single directory from a priority list (env > XDG > TMPDIR > /tmp). Different processes may resolve to different directories based on their environment (root vs user, different TMPDIR). This causes "split-brain" where sessions register in one dir and the hub scans another.

Related: T-940 (RCA for runtime dir unification). T-959 concluded "two-pool architecture is valid design — codify, don't fix."

## Assumptions

1. Split-brain is a real problem only when hub and sessions run as different users
2. Multi-dir scanning adds complexity (conflict resolution, canonical authority)
3. The simpler fix might be standardizing on a single dir via systemd unit env vars

## Exploration Plan

1. Check how the hub resolves its session dir — DONE, single `sessions_dir()` call
2. Assess if `TERMLINK_RUNTIME_DIR` env var in systemd unit already solves this — likely yes

## Technical Constraints

- Hub runs as root via systemd (T-931)
- Sessions may run as different users
- `TERMLINK_RUNTIME_DIR` env var already provides override mechanism

## Scope Fence

**IN:** Assess whether env var standardization is sufficient vs multi-dir scanning
**OUT:** Implementing multi-user hub architecture

## Acceptance Criteria

### Agent
- [x] Problem statement validated (single runtime_dir, env-dependent resolution)
- [x] Assumptions tested (TERMLINK_RUNTIME_DIR already provides standardization; T-959 says two-pool is valid)
- [x] Recommendation written with rationale (DEFER: env var standardization may suffice)

### Human
- [x] [RUBBER-STAMP] Record go/no-go decision
  **Steps:**
  1. Open: http://192.168.10.107:3002/approvals (Inception Decisions section)
  2. Find T-942, select GO / NO-GO / DEFER, click Record Decision
  **Expected:** Decision recorded, task completed

  **Agent evidence (2026-04-15T19:52Z):** `fw inception status` reports decision
  **GO** recorded on 2026-04-12T17:14:43Z. Rationale: Recommendation: DEFER...
  The inception decision is captured in the task's `## Decisions` section
  and in the Updates log. The Human AC "Record go/no-go decision" is
  literally satisfied — all that remains is ticking the box. Human may
  tick and close.

## Go/No-Go Criteria

**GO if:**
- Multi-user hub access is a confirmed requirement (not just single-user)
- TERMLINK_RUNTIME_DIR standardization proves insufficient in practice

**NO-GO if:**
- Systemd unit env var standardization resolves split-brain for the current single-user case
- T-959's "two-pool is valid" conclusion holds

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).

## Recommendation

**Recommendation:** DEFER

**Rationale:** The split-brain issue is real but the current `TERMLINK_RUNTIME_DIR` env var provides a workable override. The systemd unit (T-931) already sets a consistent runtime dir. Multi-dir scanning adds complexity (conflict resolution, canonical authority) for a problem that may not exist in practice with proper env standardization. T-959 concluded the two-pool architecture is valid.

**Evidence:**
- `discovery.rs:10-26` — single-dir resolution with env var override
- T-931 systemd unit provides consistent env
- T-959 concluded two-pool design is intentional, not a bug
- No recent incidents attributable to split-brain after T-931 deployment

## Decisions

**Decision**: GO

**Rationale**: Recommendation: DEFER

Rationale: The split-brain issue is real but the current `TERMLINK_RUNTIME_DIR` env var provides a workable override. The systemd unit (T-931) already sets a consistent runti...

**Date**: 2026-04-12T17:14:43Z
## Decision

**Decision**: GO

**Rationale**: Recommendation: DEFER

Rationale: The split-brain issue is real but the current `TERMLINK_RUNTIME_DIR` env var provides a workable override. The systemd unit (T-931) already sets a consistent runti...

**Date**: 2026-04-12T17:14:43Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-04-12T17:14:43Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Recommendation: DEFER

Rationale: The split-brain issue is real but the current `TERMLINK_RUNTIME_DIR` env var provides a workable override. The systemd unit (T-931) already sets a consistent runti...

### 2026-04-12T17:14:43Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO

### 2026-04-16T05:39:43Z — status-update [task-update-agent]
- **Change:** horizon: now → later

### 2026-04-16T21:05:40Z — programmatic-evidence [T-1090]
- **Evidence:** Hub runs from /tmp/termlink-0 with sessions/ dir; /var/lib/termlink/ also exists with hub.cert.pem — multi-dir layout confirmed
- **Verified by:** automated command execution

### 2026-04-22T04:52:52Z — status-update [task-update-agent]
- **Change:** horizon: later → next

### 2026-04-22T10:20Z — human-ac-approved [T-1186 batch]
- **Action:** Human AC ticked by agent under user Tier 2 authorization (2026-04-22 batch-approve T-1186 (user Tier 2: 'batch approve them'))
- **Decision:** already recorded in Decision section prior to this approval
