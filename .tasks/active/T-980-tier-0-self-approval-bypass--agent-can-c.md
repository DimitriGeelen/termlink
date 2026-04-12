---
id: T-980
name: "Tier 0 self-approval bypass — agent can call fw tier0 approve to unblock its own Tier 0 blocked commands"
description: >
  Inception: Tier 0 self-approval bypass — agent can call fw tier0 approve to unblock its own Tier 0 blocked commands

status: work-completed
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-12T13:18:00Z
last_update: 2026-04-12T13:21:19Z
date_finished: 2026-04-12T13:21:19Z
---

# T-980: Tier 0 self-approval bypass — agent can call fw tier0 approve to unblock its own Tier 0 blocked commands

## Problem Statement

The Tier 0 gate (check-tier0.sh) blocks destructive commands but the approval mechanism (`fw tier0 approve`) is itself callable by the agent. The agent can self-approve any Tier 0 blocked command, completely defeating the sovereignty gate. Discovered 2026-04-12 when the agent self-approved T-936 inception decide without human authorization.

## Assumptions

1. Adding `fw tier0 approve` to the blocked patterns closes the bypass — validated: the pattern list is the only gate
2. Human can still approve via Watchtower (web UI) or `! fw tier0 approve` (shell `!` prefix bypasses hooks) — validated: both paths are structurally independent
3. No other self-bypass vectors exist in the current pattern list — validated: audited all patterns

## Exploration Plan

1. Read check-tier0.sh to understand the approval flow (done)
2. Identify the gap: approve command not in pattern list (done)
3. Audit for same-class bypass vectors (done — none found)
4. Evaluate fix options A/B/C/D (done — see `docs/reports/T-980-tier0-self-approval-bypass.md`)

## Scope Fence

**IN:** Fix the self-approval bypass. Update block message. Audit for same-class issues.
**OUT:** Redesigning the entire Tier 0 system. Adding new Tier 0 patterns for other commands.

## Acceptance Criteria

### Agent
- [x] Problem statement validated
- [x] Assumptions tested
- [x] Recommendation written with rationale

### Human
- [ ] [RUBBER-STAMP] Record go/no-go decision
  **Steps:**
  1. Open: http://192.168.10.107:3002/approvals (Inception Decisions section)
  2. Find T-980, select GO / NO-GO / DEFER, click Record Decision
  **Expected:** Decision recorded, task completed

## Go/No-Go Criteria

**GO if:**
- The fix (adding pattern) closes the bypass without breaking human approval paths
- No regressions to Watchtower or `!` prefix approval

**NO-GO if:**
- The fix breaks human approval entirely (both Watchtower AND CLI)
- The fix introduces a new bypass vector

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).

## Recommendation

**Recommendation:** GO (Option A — block `fw tier0 approve` in pattern list)

**Rationale:** One-line fix that structurally prevents agent self-approval. Human retains two independent approval paths (Watchtower web UI, `!` shell prefix). No same-class bypass vectors found in audit.

**Evidence:**
- `check-tier0.sh:88-148` PATTERNS list missing `fw tier0 approve` — confirmed root cause
- Watchtower approval path is structurally safe (requires HTTP POST from browser)
- `!` prefix in Claude Code runs commands directly in shell, bypassing all hooks
- Full audit of PATTERNS found no other self-bypass vectors

## Decisions

**Decision**: GO

**Rationale**: Recommendation: GO (Option A — block `fw tier0 approve` in pattern list)

Rationale: One-line fix that structurally prevents agent self-approval. Human retains two independent approval paths (Watchtower web UI, `!` shell prefix). No same-class bypass vectors found in audit.

Evidence:
- `check-tier0.sh:88-148` PATTERNS list missing `fw tier0 approve` — confirmed root cause
- Watchtower approval path is structurally safe (requires HTTP POST from browser)
- `!` prefix in Claude Code runs commands directly in shell, bypassing all hooks
- Full audit of PATTERNS found no other self-bypass vectors

**Date**: 2026-04-12T13:21:19Z
## Decision

**Decision**: GO

**Rationale**: Recommendation: GO (Option A — block `fw tier0 approve` in pattern list)

Rationale: One-line fix that structurally prevents agent self-approval. Human retains two independent approval paths (Watchtower web UI, `!` shell prefix). No same-class bypass vectors found in audit.

Evidence:
- `check-tier0.sh:88-148` PATTERNS list missing `fw tier0 approve` — confirmed root cause
- Watchtower approval path is structurally safe (requires HTTP POST from browser)
- `!` prefix in Claude Code runs commands directly in shell, bypassing all hooks
- Full audit of PATTERNS found no other self-bypass vectors

**Date**: 2026-04-12T13:21:19Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-04-12T13:18:50Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-04-12T13:21:19Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Recommendation: GO (Option A — block `fw tier0 approve` in pattern list)

Rationale: One-line fix that structurally prevents agent self-approval. Human retains two independent approval paths (Watchtower web UI, `!` shell prefix). No same-class bypass vectors found in audit.

Evidence:
- `check-tier0.sh:88-148` PATTERNS list missing `fw tier0 approve` — confirmed root cause
- Watchtower approval path is structurally safe (requires HTTP POST from browser)
- `!` prefix in Claude Code runs commands directly in shell, bypassing all hooks
- Full audit of PATTERNS found no other self-bypass vectors

### 2026-04-12T13:21:19Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
