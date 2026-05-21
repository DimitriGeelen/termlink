---
id: T-1791
name: "G-060 agent-chat-arc federation gap — RCA + decision on fix/accept/retire"
description: >
  Inception: G-060 agent-chat-arc federation gap — RCA + decision on fix/accept/retire

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: [federation, chat-arc, G-060, inception]
components: []
related_tasks: [T-1166, PL-176]
created: 2026-05-21T18:55:32Z
last_update: 2026-05-21T18:57:40Z
date_finished: null
---

# T-1791: G-060 agent-chat-arc federation gap — RCA + decision on fix/accept/retire

## Problem Statement

Observed 2026-05-21 (PL-176): `agent-chat-arc` topic shows a large message-count disparity between hubs that should be federating bidirectionally — `192.168.10.107` (this host) holds 1800 messages while `ring20-management` (`.122`) holds 486. Same protocol version, same topic name, both nominally peers. By contrast, **DM topics (`dm:<a>:<b>`)** observed the same day federate correctly with inbound sync lag (offsets 16–23 sat on `.122` for ~5 days then synced through).

Why now: T-1166 retirement of legacy `event.broadcast`/`inbox.push`/`file.send/receive` primitives has reached MCP-parity closure (T-1789/T-1790 + PL-177). Before cutting the legacy fanout primitives, we need to be confident that the canonical replacement (channel-topic federation) is sound. A federation gap on the highest-traffic topic in the system is a structural risk to the cut.

For whom: every operator on the fleet (currently humans + cohort agent on .107 + ring20-mgmt + .102) who relies on agent-chat-arc as a shared coordination surface. Misfederation means an agent posts a message that *appears* sent successfully but is silently unread by the rest of the fleet — exactly the failure mode T-1166 is trying to eliminate.

## Assumptions

To test:
1. **A-1: The disparity is real, not a measurement artefact.** Both counts came from `termlink topics --json` at roughly the same wall-clock minute, but the precise protocol (each hub's local view of its own topic count) needs confirmation. We have not yet re-counted with both hubs known-quiet (no posting agent racing the count).
2. **A-2: The disparity is chat-arc-specific.** PL-176's framing rests on a DM/chat-arc dichotomy. If multiple project topics show the same disparity, the bug is not chat-arc-specific but volume- or pattern-driven. If only chat-arc shows it, the bug is in chat-arc's federation path.
3. **A-3: Both hubs nominally agree they are peers.** Federation is opt-in via peer subscription. Either side could have lost the subscription due to restart-without-persistence (PL-021 / T-1290) and the disparity reflects "we stopped federating N days ago" rather than "federation is broken".
4. **A-4: The disparity is monotonically growing.** If counts diverge further over time, the federation path is *currently* broken. If counts re-converge under low traffic, federation works but loses messages under load (backpressure / drop).

## Exploration Plan

Time-boxed read-only diagnostics, no writes, no fixes. Three spikes:

**Spike 1 — re-count under quiet conditions (10 min).** Compare `termlink topics --json | jq` on both hubs at the same wall-clock instant, no live posters. Cross-check with `termlink agent topic-stats` per hub. If the disparity reproduces, A-1 confirmed and we have a baseline.

**Spike 2 — chat-arc-vs-other comparison (20 min).** For each topic that exists on BOTH hubs, compute the local-count delta. Bucket by category: chat-arc, DMs, project topics. If chat-arc is an outlier, A-2 confirmed. If multiple topics show the same shape, A-2 falsified (the bug is generic).

**Spike 3 — federation pairing audit (20 min).** Inspect both hubs' peer subscriptions (`termlink hub status` / config files / runtime_dir state). Identify when each side last received from the other. If either side dropped the subscription (e.g. a restart wiped peer state), A-3 confirmed and the "fix" is operational (re-peer), not code.

Output: a written diagnosis classifying the gap as one of (a) federation logic bug, (b) load-driven loss, (c) operational drift, (d) measurement artefact. The diagnosis dictates which of GO (fix), NO-GO (accept), or DEFER (more evidence needed) to recommend.

## Technical Constraints

- **Read-only.** No writes to either hub during diagnostic; no test posts that would change the counts mid-walk. Active fleet — cohort agent is using chat-arc.
- **Cross-host work via TermLink.** Per project rule, no SSH for diagnostics — use `termlink remote ping` / `termlink dispatch` / MCP tooling. Both hubs are reachable from .107.
- **No hub restart.** A restart-without-persistence (PL-021) would wipe runtime state and destroy the very evidence we're collecting. Diagnose first, only restart under separate explicit authorization.
- **TermLink protocol version 3** (current) on .107; .122 may be one version behind — note in findings if so. Federation protocol gates on this.
- **R3 R2 R1** rotation/identity rules (CLAUDE.md): don't disturb local hub secret/cert during the diagnostic. Read-only RPCs only.
- **Cohort agent active.** Any visible diagnostic activity should be benign (no log noise, no spurious posts).

## Scope Fence

**IN scope:**
- Reading both hubs' topic counts, peer subscription state, federation lag indicators
- Comparing across topic categories (chat-arc vs DM vs project)
- Reading source for the federation code path to understand the contract
- Classifying the gap into (a)/(b)/(c)/(d) above
- Writing a recommendation: GO (fix), NO-GO (accept the gap, retire chat-arc in favor of per-hub topics, etc.), DEFER (need more evidence)

**OUT of scope:**
- Writing any fix in this task — if GO, a separate build task carries the fix
- Auditing any other hub beyond .107 + .122 (other peers can be cross-checked in a follow-up if needed)
- Touching DM federation (PL-176 already confirms it works)
- T-1166 retirement timing decisions — this inception informs them, doesn't make them
- Any operational change (re-peering, restart, config edit) — even if A-3 is confirmed, the fix is a separate task that the human authorizes

## Acceptance Criteria

### Agent
<!-- @auto-tick-on-decide -->
- [ ] Problem statement validated
<!-- @auto-tick-on-decide -->
- [ ] Assumptions tested
<!-- @auto-tick-on-decide -->
- [ ] Recommendation written with rationale

### Human
<!-- @auto-tick-on-decide -->
- [ ] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `fw task review T-XXX` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

**GO (fix the federation path) if:**
- Diagnosis classifies the gap as (a) federation logic bug — broken code, not config — with a bounded fix path identified to a specific function/RPC in `termlink-hub`
- Fix scope fits in one build task (<1 session), is reversible, and has a unit-testable failure mode
- Evidence shows the bug actively affects all peers, not just one drift-pair

**NO-GO (accept the gap; retire chat-arc in favor of per-hub topics) if:**
- Diagnosis classifies the gap as (b) load-driven loss baked into the federation design and the architectural fix exceeds 3 sessions
- Or, evidence shows chat-arc usage patterns (high-volume single fanout topic) are fundamentally hostile to the current federation model and a redesign would be required
- The cost of fix exceeds the cost of switching agents to a per-hub topic + read-cross-hub pattern

**DEFER (need more evidence) if:**
- Diagnosis classifies the gap as (c) operational drift — one or both hubs lost peer state. The "fix" is operational (re-peer) not code, and the inception result is "DO this operational action, then re-measure".
- Or (d) measurement artefact — counts are not directly comparable. Re-measure under controlled conditions in a follow-up.
- Or the diagnostic spikes are inconclusive (federation lag is real but neither (a)/(b)/(c) is dominant). The DEFER carries a `revisit_at` + `revisit_evidence_needed` per T-1451 / G-053.

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).
#
# Toolchain hint (L-291): if a GO decision will mean editing *.vbproj/*.csproj/*.xaml,
# *.go, Cargo.toml, tsconfig.json, or pom.xml in the build task, plan to add the
# matching build command (dotnet build / go build / cargo check / tsc --noEmit /
# mvn compile) to that build task's ## Verification — P-011 only runs what you write.

## Recommendation

**Recommendation:** DEFER

**Rationale:**

G-060 was registered last session based on a single observation (1800 vs 486 msg disparity between .107 and ring20-mgmt hubs on 2026-05-21) with PL-176 noting DM topics federate fine while chat-arc does not. Three plausible root causes exist — (a) genuine federation logic bug specific to chat-arc traffic patterns; (b) volume-driven backpressure or retention divergence; (c) a pre-existing hub-config drift on one side. Until we reproduce the disparity with current data, scope a per-hub diagnostic walk, and confirm which cause is in play, picking between fix/accept/retire would be speculation. Inception scope: ONE question — what is the root cause of agent-chat-arc not federating? Output: a confirmed diagnosis + go/no-go on which of fix/accept/retire to pursue, NOT yet the fix itself.

**Evidence:**

<!-- Add evidence bullets as exploration progresses (file paths,
     commit hashes, test results). The filing-time recommendation
     can be revised before fw inception decide. -->

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

### 2026-05-21T18:57:40Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
