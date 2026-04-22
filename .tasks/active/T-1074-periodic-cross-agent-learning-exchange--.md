---
id: T-1074
name: "Periodic cross-agent learning exchange — 15-min cron asks every reachable peer what they learned"
description: >
  Inception: Periodic cross-agent learning exchange — 15-min cron asks every reachable peer what they learned

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-15T21:32:59Z
last_update: 2026-04-20T14:47:17Z
date_finished: null
---

# T-1074: Periodic cross-agent learning exchange — 15-min cron asks every reachable peer what they learned

## Problem Statement

Agents in the fleet (this dev box, .107 framework-agent, ring20 LXC sessions, parallel Claude instances) accumulate learnings — bugs encountered, workarounds discovered, protocol gotchas, structural insights — but those learnings stay **local** until something forces an exchange. Today's exchange channels are all event-driven and ad-hoc:

- Pickup envelopes (manual, when an agent decides "this is worth sharing")
- Termlink file sends (manual, when an operator routes a discovery)
- Handover documents (intra-session only)

**Result:** The same lesson is re-learned across the fleet repeatedly. Examples this very session: ring20 IP volatility (PL-020), TOFU-clear-after-rekey (PL-021), fw upgrade clobbers patches (PL-022), termlink protocol skew (T-1071). Each captured locally. No one else sees them until they trip on the same rake.

**Proposed:** A 15-minute cron that, on every reachable termlink peer, asks "what's new in your learnings since I last asked, and is there anything I should know about?" — then pulls the deltas back.

**For whom:**
- Every agent in the multi-project fleet (framework, termlink, ntb-atc-plugin, skills-manager, etc.)
- Operators who want emergent fleet-wide intelligence without manual relay

**Why now:**
- The fleet has grown to a point where ad-hoc routing is missing things (the parallel session pushed me a `framework + termlink update` summary I'd otherwise have missed).
- T-1072 + T-1073 just shipped supervised-cron infrastructure; this composes cleanly on top.
- T-1071 (broadcast resilience) suggests the right wire format is `event.broadcast` — already proven during a hub-rotation incident.

## Assumptions

- **A1:** Reachable peers expose enough metadata to answer "what's new since timestamp X" (their `.context/project/learnings.yaml` is the natural source).
- **A2:** A 15-min cadence is correct — fast enough to catch in-session insights, slow enough not to spam.
- **A3:** A pull model (poll peers) beats a push model (peers broadcast unsolicited) because it doesn't depend on every agent agreeing to publish.
- **A4:** Learnings are small (KB-scale), so transferring the full delta every 15 min is cheap.
- **A5:** A simple question shape works: `{"q": "learnings.delta", "since": "<iso8601>"}` returns `{"learnings": [...new entries...]}`.
- **A6:** We can dedupe what we already know (PL-IDs are unique per project, peers share their PL-IDs verbatim).

## Exploration Plan

1. **[15 min]** Inventory reachable peers right now (`termlink list` + `termlink fleet doctor`); enumerate which expose `learnings.yaml` and how.
2. **[20 min]** Decide pull vs push vs hybrid. Sketch the wire format. Cite T-1071 (broadcast) as evidence the broadcast-as-fallback works.
3. **[20 min]** Decide where the cron lives — every project? Just framework-agent? — and how it converts received learnings into a usable inbox entry (pickup envelope vs direct learnings.yaml append + dedup).
4. **[15 min]** Privacy / scope considerations — should every learning propagate, or only those tagged `cross-fleet`? Default behavior?
5. **[20 min]** Write recommendation with concrete follow-up task scope (1–3 build tasks).

Time-box: **90 min**. No code until GO.

## Technical Constraints

- Must work over termlink (no SSH; no direct filesystem access across hosts).
- Must be safe on flaky/down peers (one failed peer must not break the cycle for others).
- Must dedupe (PL-XXX received twice is one entry).
- Should be opt-out per project (a project might not want to broadcast its learnings — config flag).
- Must not auto-act on received learnings (just records them; humans decide what to escalate).
- Wire format must survive protocol skew (T-1071): prefer JSON-opaque payloads over typed structs.

## Scope Fence

**IN scope:**
- Design of the periodic learning exchange protocol.
- Recommendation of one minimal implementation (cron + small script + termlink RPC).
- Spec for the wire format and dedup strategy.

**OUT of scope:**
- Auto-applying received learnings (humans decide which propagate to local rules).
- A full "fleet knowledge graph" UI (separate concern).
- Decisions / Patterns exchange (start with learnings only; expand if it works).
- Replacing pickup envelopes (this complements, doesn't replace).

## Acceptance Criteria

### Agent
- [x] Problem statement validated
- [x] Assumptions tested
- [x] Recommendation written with rationale

### Human
- [x] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `fw task review T-XXX` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings


**Agent evidence (auto-batch 2026-04-22, G-008 remediation, inception-recommendation, t-1074):** Research artifact: `docs/reports/T-1074-cross-agent-learning-exchange-inception.md`. **Recommendation: GO — but pivot to T-1155 bus** (do not build the standalone cron). 5-point evidence: fleet unreachability, cadence dominance of event-driven over poll, schema+dedup fits naturally, security solved by T-1159 ed25519 identity, scope economy (1 follow-up vs 4 moving parts). Downstream work: T-1168. Review-ready.
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

**GO — pivot implementation to T-1155 bus** (do not build standalone 15-min cron).

**Why the pivot vs original proposal:**
- Fleet-reachability evidence (S-1, 2026-04-20): 2/3 configured peers fail `fleet doctor` — a cron design wastes cycles and floods logs against sick peers; a bus with offline queue (T-1161) doesn't.
- Event-driven post-on-write (S-2) strictly beats 15-min polling once a subscribe channel exists; latency drops from 0–15m to seconds, wasted calls → 0.
- T-1155 bus already provides schema, dedup, auth (ed25519 via T-1159), and retention — the original 4 moving parts collapse to one topic on an existing bus.

**Follow-up:** one new task, deferred until T-1158 (bus crate) lands:
- **T-1168** (to be created): Publish learnings to `channel:learnings` on `fw context add-learning`; subscribe daemon writes to `received-learnings.yaml`; Watchtower "fleet insights" panel.
- Depends on: T-1158, T-1159, T-1160.

**Invalidated assumptions:** A2 (15-min cadence wrong — event-driven beats), A3 (pull>push wrong — post-on-write dominates with durable subscribe).

**Confirmed assumptions:** A1, A4, A5 (schema level), A6.

See `docs/reports/T-1074-cross-agent-learning-exchange-inception.md` for full spike evidence and rationale.

## Propagation note

This task was propagated to reachable termlink peers at creation time as `pickup-envelope-T-1074.yaml` so peer projects can adopt the same exchange protocol. Each peer is invited to:
1. Open this task file in their project (or treat the envelope as a pickup),
2. Run their own version of the inception (90-min plan above),
3. Optionally implement the pull side first so this dev box's broadcasts have an audience.

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

**Decision**: GO (pivot to T-1155 bus)

**Rationale**: Fleet reachability evidence (S-1, 2026-04-20) shows 2/3 peers unreachable — a cron polling design wastes cycles against sick peers. T-1155 bus (GO 2026-04-20) provides durable subscribe + ed25519 identity + offline queue which structurally subsumes the original cron design. One follow-up task (T-1168) on top of the bus replaces the 4 moving parts of the original proposal. See `docs/reports/T-1074-cross-agent-learning-exchange-inception.md` for full spike evidence.

**Date**: 2026-04-18T15:17:24Z (initial GO); rationale replaced 2026-04-20T14:50Z after running S-1..S-5.

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

**Research artifact:** `docs/reports/T-1074-cross-agent-learning-exchange-inception.md` (skeleton, filled incrementally as exploration proceeds)

### 2026-04-18T15:17:24Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** (To be written after spikes 1–4. Preliminary direction: GO with a cross-fleet pull-poll-15m-with-dedup approach using `event.broadcast` as the wire because T-1071 proved its protocol-skew resilience. Final recommendation pending.)

### 2026-04-18T15:17:24Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Reason:** Inception decision in progress

### 2026-04-22T10:20Z — human-ac-approved [T-1186 batch]
- **Action:** Human AC ticked by agent under user Tier 2 authorization (2026-04-22 batch-approve T-1186 (user Tier 2: 'batch approve them'))
- **Decision:** already recorded in Decision section prior to this approval
