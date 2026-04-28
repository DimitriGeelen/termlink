---
id: T-1384
name: "Are we ready to push agent-conversation arc to other agents (cross-machine), and what end-to-end multi-agent test should we run with the fleet we already have?"
description: >
  Inception: Are we ready to push agent-conversation arc to other agents (cross-machine), and what end-to-end multi-agent test should we run with the fleet we already have?

status: work-completed
workflow_type: inception
owner: human
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/channel.rs, crates/termlink-session/src/bus_client.rs]
related_tasks: []
created: 2026-04-28T17:14:57Z
last_update: 2026-04-28T19:33:22Z
date_finished: 2026-04-28T19:33:22Z
---

# T-1384: Are we ready to push agent-conversation arc to other agents (cross-machine), and what end-to-end multi-agent test should we run with the fleet we already have?

## Problem Statement

We have shipped a 59-task agent-conversation arc (T-1325 → T-1383, Matrix
primitives mirrored onto termlink topics). Every test has run on a single
local hub against synthetic identities. Two unanswered questions:
(1) is the arc ready to push to the agents in our actual fleet, and
(2) given the fleet we have today, what's the strongest end-to-end test
we can run? Full inception artifact: `docs/reports/T-1384-multi-agent-readiness-inception.md`.

## Assumptions

- A1: ≥3 hub-bearing agents reachable — **PARTIAL** (2 reachable: local + .122; .121 auth-broken)
- A2: Remote hubs have new T-1376..T-1383 commands — **FALSE** (.122 hub at 0.9.844, no `channel.*` RPCs)
- A3: Cross-hub canonical state convergence works — **BLOCKED** by A2
- A6: New binary deploy via termlink primitives works — **FALSE** (legacy file-event fallback didn't auto-install)

## Exploration Plan

- S1 fleet inventory — done
- S2 cross-hub via .122 — done, BLOCKED finding
- S3a re-run e2e on system binary — done, passed
- S3b 3-session local multi-post test — done, revealed identity gap
- S4–S5 multi-machine variant — DEFERRED (blocked by remote hub version)

## Technical Constraints

- Hub `channel.*` RPCs require client+hub at ≥0.9.1xxx. Remote .122 hub at 0.9.844; deploy + restart needed.
- Identity is per-user (per `~/.termlink/identity/`), not per-session — multiple Claude Code sessions on one host share one conversational identity.
- SSH key auth to .121 not configured → blocks `fleet reauth ring20-dashboard --bootstrap-from ssh:`

## Scope Fence

IN: inventory, version census, cross-hub spike, local multi-session spike, GO/NO-GO/DEFER decision.
OUT: hub restart on .122 (operational, human-supervised); per-session identity build (T-1385); SSH heal for .121 (T-1387).

## Acceptance Criteria

### Agent
- [x] Problem statement validated
- [x] Assumptions tested (A1=PARTIAL, A2=FALSE, A3=BLOCKED, A6=FALSE)
- [x] Recommendation written with rationale

### Human
- [x] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `fw task review T-XXX` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

**GO (push to fleet now) if:** ≥3 reachable agents all running version with `channel.*` RPCs, and cross-hub canonical state byte-identical between any two of them.

**NO-GO (block arc rollout) if:** version skew requires hub-side rewrite (not just rebuild); OR canonical state diverges across observers.

**DEFER (capture findings, follow up later) if:** version skew is "rebuild + redeploy + restart" — solvable by ops work, not arc-design work — AND the local-host arc itself works correctly. **This is the actual situation observed.**

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).

## Recommendation

**Recommendation:** DEFER full fleet rollout. **GO** on the arc itself for local-host multi-identity scenarios (already validated). Open three follow-up tasks for the gaps.

**Rationale:** The arc is correct (531 unit tests + 55-step e2e green twice this session). What blocks fleet rollout is purely operational/architectural, not a flaw in the arc design:

1. Remote hub at .122 lacks `channel.*` RPCs because it runs 0.9.844 — needs binary deploy + hub restart (service-impacting, human-supervised).
2. Multi-session-on-one-host produces ONE identity, not many — per-user not per-session — needs an opt-in mechanism (build task).
3. ring20-dashboard heal is SSH-blocked — needs Tier-2 reauth or SSH key setup.

Any of these is a clean, scoped follow-up task. None requires arc redesign.

**Evidence:**
- S1: 2 reachable hubs (.107=local, .122=ring20-management); .121 auth-broken; .107 = this machine with 14 live sessions
- S2: cross-hub `channel create` against .122 fails with RPC error — remote hub lacks the namespace at version 0.9.844
- S3a: 55-step e2e green twice this session (cargo binary + system binary at /usr/local/bin/termlink)
- S3b: 3-session local-hub posting works at envelope level; both successful posts came from `sender_id=d1993c2c3ec44c94` (same identity) — architectural finding: identity is per-user not per-session
- Local /usr/local/bin/termlink upgraded 0.9.844 → 0.9.1527 with safety backup at `.0.9.844.bak`; verified all 14 sessions now resolve to new binary

**Follow-up tasks:**
- T-1385 (build): per-session identity opt-in
- T-1386 (deploy): binary deploy + hub restart on .122 + .121
- T-1387 (build, optional): Tier-2 reauth without SSH-key dependency

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

**Rationale**: Cross-hub gap closed in T-1385. Multi-machine + live-agent e2e proven in T-1386/T-1387.
  Arc is ready to push to the fleet.

**Date**: 2026-04-28T19:33:22Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-04-28T17:16:31Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-04-28T18:46:06Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** DEFER
- **Rationale:** Recommendation: DEFER full fleet rollout. GO on the arc itself for local-host multi-identity scenarios (already validated). Open three follow-up tasks for the gaps.

Rationale: The arc is correct (531 unit tests + 55-step e2e green twice this session). What blocks fleet rollout is purely operational/architectural, not a flaw in the arc design:

1. Remote hub at .122 lacks `channel.` RPCs because it runs 0.9.844 — needs binary deploy + hub restart (service-impacting, human-supervised).
2. Multi-session-on-one-host produces ONE identity, not many — per-user not per-session — needs an opt-in mechanism (build task).
3. ring20-dashboard heal is SSH-blocked — needs Tier-2 reauth or SSH key setup.

Any of these is a clean, scoped follow-up task. None requires arc redesign.

Evidence:
- S1: 2 reachable hubs (.107=local, .122=ring20-management); .121 auth-broken; .107 = this machine with 14 live sessions
- S2: cross-hub `channel create` against .122 fails with RPC error — remote hub lacks the namespace at version 0.9.844
- S3a: 55-step e2e green twice this session (cargo binary + system binary at /usr/local/bin/termlink)
- S3b: 3-session local-hub posting works at envelope level; both successful posts came from `sender_id=d1993c2c3ec44c94` (same identity) — architectural finding: identity is per-user not per-session
- Local /usr/local/bin/termlink upgraded 0.9.844 → 0.9.1527 with safety backup at `.0.9.844.bak`; verified all 14 sessions now resolve to new binary

Follow-up tasks:
- T-1385 (build): per-session identity opt-in
- T-1386 (deploy): binary deploy + hub restart on .122 + .121
- T-1387 (build, optional): Tier-2 reauth without SSH-key dependency

### 2026-04-28T19:33:22Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Cross-hub gap closed in T-1385. Multi-machine + live-agent e2e proven in T-1386/T-1387.
  Arc is ready to push to the fleet.

### 2026-04-28T19:33:22Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
