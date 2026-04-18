---
id: T-1101
name: "Deep value assessment — what does termlink actually give the human operator, what works end-to-end, what's cargo cult"
description: >
  Inception: Deep value assessment — what does termlink actually give the human operator, what works end-to-end, what's cargo cult

status: work-completed
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-17T08:34:06Z
last_update: 2026-04-17T10:50:39Z
date_finished: 2026-04-17T10:50:39Z
---

# T-1101: Deep value assessment — what does termlink actually give the human operator, what works end-to-end, what's cargo cult

## Problem Statement

We've built 68 MCP tools, 1,124 tests, fleet management, and deep infrastructure.
But the human operator's daily experience is fragmented. The research artifact at
`docs/reports/T-1101-termlink-value-assessment.md` documents the full findings.

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
- [x] Problem statement validated — research artifact with 5 spikes
- [x] Assumptions tested — live fleet evidence gathered
- [x] Recommendation written with rationale
- [x] First build task (T-1102: fleet status) executed as proof of value

### Human
- [ ] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `fw task review T-XXX` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

<!-- Fill these BEFORE writing the recommendation. The placeholder detector will block review/decide if left empty. -->
**GO if:**
- Assessment identifies operator-facing features that can be built on existing architecture
- At least one feature can be shipped same-session as proof of value

**NO-GO if:**
- Architecture fundamentally can't serve operator needs
- All identified features require major redesign

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).

## Recommendation

**Recommendation:** GO

**Rationale:** The architecture is solid — hub RPC, fleet doctor, auth, discovery all work.
The gap is in the presentation and operator experience layer. T-1102 (fleet status) was
built and shipped in one session as proof that high-value operator features can be
built quickly on the existing foundation.

**Evidence:**
- Fleet doctor correctly diagnoses .121 auth-fail and .122 hub-down (2 days running)
- 36 local sessions, 10 hub-registered sessions — discovery works but lacks summary view
- `termlink fleet status` built in ~30 min, provides the "morning check" the operator needs
- No VPN/mesh test tooling exists — identified as highest-value gap
- Watchtower has 12 pages but none are operations-focused (all framework-focused)

**Priority build queue:**
1. ~~R1: `fleet status`~~ DONE (T-1102)
2. R2: Watchtower `/fleet` page — operations dashboard
3. R3: `termlink net test` — mesh connectivity (needs new RPC method)
4. R4: Fix .121/.122 fleet health (SSH required, human action)
5. R5: Clickable references in Watchtower pages

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

Rationale: The architecture is solid — hub RPC, fleet doctor, auth, discovery all work.
The gap is in the presentation and operator experience layer. T-1102 (fleet status) was
built and shipped in one session as proof that high-value operator features can be
built quickly on the existing foundation.

Evidence:
- Fleet doctor correctly diagnoses .121 auth-fail and .122 hub-down (2 days running)
- 36 local sessions, 10 hub-registered sessions — discovery works but lacks summary view
- `termlink fleet status` built in ~30 min, provides the "morning check" the operator needs
- No VPN/mesh test tooling exists — identified as highest-value gap
- Watchtower has 12 pages but none are operations-focused (all framework-focused)

Priority build queue:
1. ~~R1: `fleet status`~~ DONE (T-1102)
2. R2: Watchtower `/fleet` page — operations dashboard
3. R3: `termlink net test` — mesh connectivity (needs new RPC method)
4. R4: Fix .121/.122 fleet health (SSH required, human action)
5. R5: Clickable references in Watchtower pages

**Date**: 2026-04-17T10:50:38Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-04-17T08:37:52Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-04-17T10:50:38Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Recommendation: GO

Rationale: The architecture is solid — hub RPC, fleet doctor, auth, discovery all work.
The gap is in the presentation and operator experience layer. T-1102 (fleet status) was
built and shipped in one session as proof that high-value operator features can be
built quickly on the existing foundation.

Evidence:
- Fleet doctor correctly diagnoses .121 auth-fail and .122 hub-down (2 days running)
- 36 local sessions, 10 hub-registered sessions — discovery works but lacks summary view
- `termlink fleet status` built in ~30 min, provides the "morning check" the operator needs
- No VPN/mesh test tooling exists — identified as highest-value gap
- Watchtower has 12 pages but none are operations-focused (all framework-focused)

Priority build queue:
1. ~~R1: `fleet status`~~ DONE (T-1102)
2. R2: Watchtower `/fleet` page — operations dashboard
3. R3: `termlink net test` — mesh connectivity (needs new RPC method)
4. R4: Fix .121/.122 fleet health (SSH required, human action)
5. R5: Clickable references in Watchtower pages

### 2026-04-17T10:50:39Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
