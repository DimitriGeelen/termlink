---
id: T-942
name: "Pickup: Hub should scan multiple session dirs — eliminates runtime dir split-brain (from termlink)"
description: >
  Auto-created from pickup envelope. Source: termlink, task T-940. Type: feature-proposal.

status: started-work
workflow_type: inception
owner: agent
horizon: now
tags: [pickup, feature-proposal]
components: []
related_tasks: []
created: 2026-04-12T07:49:08Z
last_update: 2026-04-12T07:49:08Z
date_finished: null
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
- [ ] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `fw task review T-XXX` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

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
