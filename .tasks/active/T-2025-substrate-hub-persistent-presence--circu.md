---
id: T-2025
name: "Substrate: hub-persistent presence + circuit-breaker state across restarts"
description: >
  §6 primitive 7 (Resilience, hard-dep). Presence and circuit-breaker state are in-memory
  today; reset on hub restart, so liveness inference resets to everyone-unknown for
  one heartbeat interval after every restart. Channel logs + inbox spool DO survive,
  so message durability is intact; only the liveness picture is fragile.

status: captured
workflow_type: inception
owner: human
horizon: later
tags: [arc:arc-parallel-substrate]
components: []
related_tasks: [T-2018]
created: 2026-06-07T11:36:42Z
last_update: '2026-06-07T11:41:30Z'
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
  confidence: 0
  disposition: deferred
  rationale: captured-while-fresh per [[PL-203]]; design-phase decision — hint: Co-locating may simplify ops but couples concerns

- **IW-2: TTL semantics — STALE vs OFFLINE thresholds, recoverable vs evicted?**
  confidence: 0
  disposition: deferred
  rationale: captured-while-fresh per [[PL-203]]; design-phase decision — hint: Default thresholds matter for ring20 partition behavior

- **IW-3: Circuit-breaker scope — per-spoke, per-topic, per-target-host?**
  confidence: 0
  disposition: deferred
  rationale: captured-while-fresh per [[PL-203]]; design-phase decision — hint: Scope choice affects fail-isolation granularity

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

**Recommendation:** DEFER

**Rationale:** Captured-while-fresh per PL-203; per-primitive design follows operator promotion.

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
