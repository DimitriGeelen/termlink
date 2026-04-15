---
id: T-1071
name: "Framework improvements from termlink protocol-skew + event.broadcast workaround pattern"
description: >
  Inception: Framework improvements from termlink protocol-skew + event.broadcast workaround pattern

status: captured
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-15T21:18:07Z
last_update: 2026-04-15T21:18:07Z
date_finished: null
---

# T-1071: Framework improvements from termlink protocol-skew + event.broadcast workaround pattern

## Problem Statement

A parallel session observation (2026-04-15T21:14Z, relayed from ring20-dashboard on .121):

> "PTY inject/exec not possible right now due to termlink protocol skew: our client 0.9.844 sends keys as plain string, but .107's newer hub expects adjacently tagged KeyEntry struct — so command.inject and command.exec both fail with parse errors. The workaround (event.broadcast via remote_call) landed cleanly and will show up on every session's event bus."

**What this reveals (candidates to learn from, pending exploration):**

1. **Protocol skew between clients and hubs is normal and recurring.** `fw upgrade` is not idempotent for binaries (T-1070). When a hub is newer than its clients, some RPC methods break silently while others keep working.
2. **`event.broadcast` proved to be the resilience valve.** When point-to-point methods failed due to schema drift, the broadcast path still worked and reached 12/12 sessions. That's a load-bearing property worth codifying.
3. **There is no structural "protocol version negotiation" between client and hub.** Errors surface as parse failures, not as actionable "upgrade your client" messages.
4. **Observability of this class of failure was agent-driven.** The framework didn't flag "your fleet has 3 distinct termlink versions in circulation." The agent inferred it from one failed RPC.

**For whom:** operators running heterogeneous termlink versions across a fleet (the normal state); agents coordinating across such fleets; framework maintainers considering how to promote resilience patterns to first-class structural features.

**Why now:** The workaround worked today; it won't always. The next skew may not have a broadcast fallback. Capture the learning while the evidence is fresh.

## Assumptions

- **A1:** The protocol-skew failure mode is repeatable — every client/hub version pair on a divergent schema will show it.
- **A2:** `event.broadcast` is structurally more resilient to schema drift than typed RPCs because its payload is opaque JSON, not a typed struct.
- **A3:** The framework *could* warn on fleet-wide version skew — `fleet doctor` already connects to each hub.
- **A4:** This is a framework-level concern, not a termlink-only concern — any cross-version RPC is vulnerable.

## Exploration Plan

1. **[15 min]** Confirm the failure mode via code: grep for `KeyEntry` in termlink + check git log for the schema change.
2. **[15 min]** Audit which termlink RPC methods are schema-opaque (resilient) vs. typed (fragile). Short table.
3. **[20 min]** Inventory what the framework could learn — protocol version negotiation, fleet-wide version reporting in `fleet doctor`, a "resilience tier" label on RPC methods, auto-warn on skew.
4. **[20 min]** Decide: (a) termlink-only fixes, (b) framework-level observability, or (c) both. Formulate 1–3 follow-up task scopes.

Total time-box: **70 minutes**. No code until GO.

## Technical Constraints

- Any client→hub protocol negotiation must be backwards compatible.
- Version data for fleet skew detection must be cheap to collect (piggy-back on existing `doctor` pings).
- Must not regress the property that actually worked today: `event.broadcast` succeeding despite schema drift.

## Scope Fence

**IN scope:**
- Identify concrete framework-level lessons from the skew+broadcast pattern.
- Recommend 1–3 follow-up tasks (observability, negotiation, or resilience-tier taxonomy).

**OUT of scope:**
- Implementing the fix (would be separate build tasks after GO).
- Fixing the specific `KeyEntry` schema (that's a termlink build task, out of framework scope).
- Propagating patches upstream (that's T-1069 territory).

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

<!-- Fill these BEFORE writing the recommendation. The placeholder detector will block review/decide if left empty. -->
**GO if:**
- Root cause identified with bounded fix path
- Fix is scoped, testable, and reversible

**NO-GO if:**
- Problem requires fundamental redesign or unbounded scope
- Fix cost exceeds benefit given current evidence

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
