---
id: T-2025
name: "Substrate: hub-persistent presence + circuit-breaker state across restarts"
description: >
  §6 primitive 7 (Resilience, hard-dep). Presence and circuit-breaker state are in-memory
  today; reset on hub restart, so liveness inference resets to everyone-unknown for
  one heartbeat interval after every restart. Channel logs + inbox spool DO survive,
  so message durability is intact; only the liveness picture is fragile.

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: [arc:arc-parallel-substrate]
components: []
related_tasks: [T-2018]
created: 2026-06-07T11:36:42Z
last_update: 2026-06-08T07:28:22Z
date_finished:
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 3            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.5                    # float 0..1. Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
bvp_scores_proposed:
  - ts: '2026-06-07T11:41:30Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 2
      D2: 2
      D3: 2
      D4: 2
    rationale: D1=2 (no-signal); D2=2 (no-signal); D3=2 (no-signal); D4=2 
      (no-signal)
    rubric_sha: missing
---

# T-2025: Substrate: hub-persistent presence + circuit-breaker state across restarts

## Problem Statement

§6 primitive of [arc-parallel-substrate](../../docs/architecture/parallel-execution-substrate.md). **Group:** Resilience. **§9 boundary:** hard-dep per §9.

**Role per ADR §6:** Presence + circuit-breaker state are in-memory today; reset on hub restart. Liveness inference resets to 'everyone unknown' for one heartbeat interval after every restart. Channel logs + inbox spool DO survive — only the *liveness picture* is fragile.

**Why captured now:** T-2020 (registry) needs this to be load-bearing. Filing now while the restart-failure-mode is in view.

**Status disclosure ([[PL-203]]):** Filed `status: captured, horizon: later`. BVP scores will be estimator-proposed (not confirmed) until promotion. No commitment to RPC shape, build order, or cost-estimate is locked at filing. Per-primitive design phase opens at operator promotion via `fw bvp confirm` + `--horizon now`.

## Assumptions

<!-- Key assumptions to test. Register with: fw assumption add "Statement" --task T-XXX -->

## Open Questions

- **IW-1: Storage — SQLite alongside channel logs, or a separate keyed store?**
  confidence: 4
  disposition: resolved
  rationale: MOOT. No new storage needed. The `agent-presence` topic IS already SQLite-backed and durable (retention=forever on the live hub; 13441 posts persisted). Heartbeats flow through `Bus::post()` (`crates/termlink-bus/src/lib.rs:127`), which is the same path as every other channel. Per ADR §3, channel data survives hub restart by construction. The "in-memory" framing in §6 referred to the DERIVED LIVE/STALE/OFFLINE view, not the underlying data. See docs/reports/T-2025-persistent-presence-circuit-breaker-inception.md §2.

- **IW-2: TTL semantics — STALE vs OFFLINE thresholds, recoverable vs evicted?**
  confidence: 4
  disposition: resolved
  rationale: ALREADY CLIENT-SIDE POLICY. `scripts/agent-listeners.sh` and `agent-listeners-fleet.sh` apply consumer-side thresholds (default: LIVE ≤ 35s, STALE ≤ 90s, OFFLINE otherwise). Substrate need not enforce a global TTL — different consumers (orchestrator vs handover vs operator dashboard) want different windows. Keep policy at the consumer; substrate ships the durable data. See artifact §5.IW-2.

- **IW-3: Circuit-breaker scope — per-spoke, per-topic, per-target-host?**
  confidence: 3
  disposition: resolved
  rationale: CURRENT SCOPE (per-session-id) IS WORKABLE; refinement is conditional. `crates/termlink-hub/src/circuit_breaker.rs` keys per-session-id today (which conflates with per-connection on TCP). A future per-spoke or per-topic split is a refinement that should follow real incident evidence, not speculation. More importantly: persisting circuit-breaker state across hub restart is arguably the WRONG default — restart is a recovery event, and carrying forward OPEN classifications blocks traffic to peers whose underlying issue has healed. See artifact §3.

## Exploration Plan

At promotion time: (1) decide storage location; (2) define TTL semantics; (3) implement with restart-survival test; (4) wire to T-2020 once both are ready.

## Technical Constraints

**Dependencies (upstream):** None

**Dependencies (downstream):** T-2020 (registry) — needs this to be useful across hub restart

**ADR §9 boundary:** hard-dep per §9

## Scope Fence

**IN scope (this inception):** Validate that §6's description still holds in light of what's been learned from earlier primitives. Refine open questions into a design proposal. Recommend GO / NO-GO / DEFER with rationale. Surface any newly-discovered sub-decomposition.

**OUT of scope (this inception):** Build/code work — that's a follow-on task created on GO. Other primitives' shapes — they have their own tasks. AEF orchestration layer integration — that's the §9 collaboration seam, owned at the boundary.

## Acceptance Criteria

### Agent
<!-- @auto-tick-on-decide -->
- [x] Problem statement validated
<!-- @auto-tick-on-decide -->
- [x] Assumptions tested
<!-- @auto-tick-on-decide -->
- [x] Recommendation written with rationale

### Human
<!-- @auto-tick-on-decide -->
- [x] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `fw task review T-XXX` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

**GO if:**
- Storage chosen and tested across hub restart; TTL semantics documented; T-2020 can build against this.

**NO-GO if:**
- Storage choice creates SQLite contention; restart-survival test reveals data-loss path.

**DEFER if:**
- Predecessor primitives have shifted shape in ways that change the open questions; capture the shift, update the ADR, re-file.

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

**Recommendation:** NO-GO as captured. Re-scope as documentation-only.

**Rationale (one-paragraph):** Investigation shows the captured framing does not match the running system. The `agent-presence` topic IS already durable (retention=forever, SQLite-backed, 13441 posts persisted on the live hub) — what's in-memory is the DERIVED LIVE/STALE/OFFLINE view, which is reconstructed from heartbeats on every query, not stored. The post-restart "blackout" is bounded by one heartbeat interval (~30s) of information staleness, not data loss. Circuit-breaker state IS in-memory (`crates/termlink-hub/src/circuit_breaker.rs`), but persisting it across hub restart is arguably the WRONG default — restart is a recovery event; carrying forward OPEN classifications blocks traffic to peers whose underlying issue has since healed. Building T-2025 as captured would introduce two new SQLite tables (`agent_presence_memo`, `circuit_breaker_state`) that DUPLICATE existing state with the wrong default semantics — the same "two sources of truth" anti-pattern T-2020 identified and avoided.

**Full analysis:** see [docs/reports/T-2025-persistent-presence-circuit-breaker-inception.md](../../docs/reports/T-2025-persistent-presence-circuit-breaker-inception.md).

**Action on NO-GO (documentation-only, blast_radius=0):**
- Update ADR §6 #7 description to reflect actual state ("presence DATA is durable; derived view is in-memory but reconstructible; circuit-breaker reset is intentional, not a gap").
- Add post-restart blackout paragraph to `docs/operations/substrate-claim-primitive.md` documenting that `find_idle` (T-2020) returns prior-heartbeat data immediately and refreshes within one heartbeat interval.

**GO criteria evaluation (from §Go/No-Go Criteria):**
- ❌ "Storage chosen and tested across hub restart" — no storage needed; data already durable.
- ❌ "TTL semantics documented" — already client-side policy, not substrate concern.
- ✅ (negative form) "T-2020 can build against this" — T-2020 already builds against the existing durable topic; no T-2025 dependency.

**Conditional follow-up tasks (only if real evidence surfaces):**
- *(Optimization, not primitive)* Presence-memo for sub-O(topic_size) `find_idle` queries — file with measured benchmark if T-2020 ships and `find_idle` latency becomes a problem at >30 agents.
- *(Hub-config, not primitive)* Sticky circuit-breaker flag for deployments that restart hub frequently for unrelated reasons — file with operator pain-point evidence.

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

**Decision**: NO-GO

**Rationale**: Recommendation: NO-GO as captured. Re-scope as documentation-only.

Rationale (one-paragraph): Investigation shows the captured framing does not match the running system. The `agent-presence` topic IS already durable (retention=forever, SQLite-backed, 13441 posts persisted on the live hub) — what's in-memory is the DERIVED LIVE/STALE/OFFLINE view, which is reconstructed from heartbeats on every query, not stored. The post-restart "blackout" is bounded by one heartbeat interval (~30s) of information staleness, not data loss. Circuit-breaker state IS in-memory (`crates/termlink-hub/src/circuit_breaker.rs`), but persisting it across hub restart is arguably the WRONG default — restart is a recovery event; carrying forward OPEN classifications blocks traffic to peers whose underlying issue has since healed. Building T-2025 as captured would introduce two new SQLite tables (`agent_presence_memo`, `circuit_breaker_state`) that DUPLICATE existing state with the wrong default semantics — the same "two sources of truth" anti-pattern T-2020 identified and avoided.

Full analysis: see [docs/reports/T-2025-persistent-presence-circuit-breaker-inception.md](../../docs/reports/T-2025-persistent-presence-circuit-breaker-inception.md).

Action on NO-GO (documentation-only, blast_radius=0):
- Update ADR §6 #7 description to reflect actual state ("presence DATA is durable; derived view is in-memory but reconstructible; circuit-breaker reset is intentional, not a gap").
- Add post-restart blackout paragraph to `docs/operations/substrate-claim-primitive.md` documenting that `find_idle` (T-2020) returns prior-heartbeat data immediately and refreshes within one heartbeat interval.

GO criteria evaluation (from §Go/No-Go Criteria):
- ❌ "Storage chosen and tested across hub restart" — no storage needed; data already durable.
- ❌ "TTL semantics documented" — already client-side policy, not substrate concern.
- ✅ (negative form) "T-2020 can build against this" — T-2020 already builds against the existing durable topic; no T-2025 dependency.

Conditional follow-up tasks (only if real evidence surfaces):
- (Optimization, not primitive) Presence-memo for sub-O(topic_size) `find_idle` queries — file with measured benchmark if T-2020 ships and `find_idle` latency becomes a problem at >30 agents.
- (Hub-config, not primitive) Sticky circuit-breaker flag for deployments that restart hub frequently for unrelated reasons — file with operator pain-point evidence.

**Date**: 2026-06-08T11:21:06Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-06-08T07:28:22Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)

### 2026-06-08T11:21:06Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** NO-GO
- **Rationale:** Recommendation: NO-GO as captured. Re-scope as documentation-only.

Rationale (one-paragraph): Investigation shows the captured framing does not match the running system. The `agent-presence` topic IS already durable (retention=forever, SQLite-backed, 13441 posts persisted on the live hub) — what's in-memory is the DERIVED LIVE/STALE/OFFLINE view, which is reconstructed from heartbeats on every query, not stored. The post-restart "blackout" is bounded by one heartbeat interval (~30s) of information staleness, not data loss. Circuit-breaker state IS in-memory (`crates/termlink-hub/src/circuit_breaker.rs`), but persisting it across hub restart is arguably the WRONG default — restart is a recovery event; carrying forward OPEN classifications blocks traffic to peers whose underlying issue has since healed. Building T-2025 as captured would introduce two new SQLite tables (`agent_presence_memo`, `circuit_breaker_state`) that DUPLICATE existing state with the wrong default semantics — the same "two sources of truth" anti-pattern T-2020 identified and avoided.

Full analysis: see [docs/reports/T-2025-persistent-presence-circuit-breaker-inception.md](../../docs/reports/T-2025-persistent-presence-circuit-breaker-inception.md).

Action on NO-GO (documentation-only, blast_radius=0):
- Update ADR §6 #7 description to reflect actual state ("presence DATA is durable; derived view is in-memory but reconstructible; circuit-breaker reset is intentional, not a gap").
- Add post-restart blackout paragraph to `docs/operations/substrate-claim-primitive.md` documenting that `find_idle` (T-2020) returns prior-heartbeat data immediately and refreshes within one heartbeat interval.

GO criteria evaluation (from §Go/No-Go Criteria):
- ❌ "Storage chosen and tested across hub restart" — no storage needed; data already durable.
- ❌ "TTL semantics documented" — already client-side policy, not substrate concern.
- ✅ (negative form) "T-2020 can build against this" — T-2020 already builds against the existing durable topic; no T-2025 dependency.

Conditional follow-up tasks (only if real evidence surfaces):
- (Optimization, not primitive) Presence-memo for sub-O(topic_size) `find_idle` queries — file with measured benchmark if T-2020 ships and `find_idle` latency becomes a problem at >30 agents.
- (Hub-config, not primitive) Sticky circuit-breaker flag for deployments that restart hub frequently for unrelated reasons — file with operator pain-point evidence.
