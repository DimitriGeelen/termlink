---
id: T-2028
name: "Substrate: throughput/connection budget + retention/compaction policy"
description: >
  §6 primitive 10 (Supporting). No connection cap, rate limiter, or backpressure governor
  exists. T-1991 (agent-presence bloat) was found in PRODUCTION, not predicted. The
  coordination/announcement pattern AEF wants generates exactly that traffic class,
  so retention/compaction must be designed in from the start, not bolted on.

status: captured
workflow_type: inception
owner: human
horizon: later
tags: [arc:arc-parallel-substrate]
components: []
related_tasks: [T-2018]
created: 2026-06-07T11:36:55Z
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

# T-2028: Substrate: throughput/connection budget + retention/compaction policy

## Problem Statement

§6 primitive of [arc-parallel-substrate](../../docs/architecture/parallel-execution-substrate.md). **Group:** Supporting. **§9 boundary:** cross-cutting policy — touches every primitive's design.

**Role per ADR §6:** No connection cap, rate limiter, or backpressure governor exists in code. T-1991 (agent-presence bloat to ~1800 envelopes) was found in PRODUCTION, not predicted. The coordination/announcement pattern AEF wants generates exactly that traffic class — retention/compaction must be designed in from the start, not bolted on.

**Why captured now:** Capture-while-fresh while T-1991's precedent is sharp. Likely the LAST primitive to actually build (it's a cross-cutting review), but filing it last would risk losing the policy context.

**Status disclosure ([[PL-203]]):** Filed `status: captured, horizon: later`. BVP scores will be estimator-proposed (not confirmed) until promotion. No commitment to RPC shape, build order, or cost-estimate is locked at filing. Per-primitive design phase opens at operator promotion via `fw bvp confirm` + `--horizon now`.

## Assumptions

<!-- Key assumptions to test. Register with: fw assumption add "Statement" --task T-XXX -->

## Open Questions

- **IW-1: Per-topic retention — Days(N) / Forever / message-count — confirm the existing API or add it?**
  confidence: 0
  disposition: deferred
  rationale: captured-while-fresh per [[PL-203]]; design-phase decision — hint: Check current retention API state

- **IW-2: Compaction trigger — time-based, size-based, both? Per-topic policy?**
  confidence: 0
  disposition: deferred
  rationale: captured-while-fresh per [[PL-203]]; design-phase decision — hint: T-1991 was time-based pressure

- **IW-3: Connection cap — per-process, per-host, per-hub? Behavior when hit — queue or refuse?**
  confidence: 0
  disposition: deferred
  rationale: captured-while-fresh per [[PL-203]]; design-phase decision — hint: Refusal must be loud (G-058-style silent failures are the enemy)

- **IW-4: Rate limit — per-sender, per-topic, per-RPC? Budget visible to clients in `topic info`?**
  confidence: 0
  disposition: deferred
  rationale: captured-while-fresh per [[PL-203]]; design-phase decision — hint: Observability matters; clients should see the budget

- **IW-5: T-1991 precedent — what was the would-have-helped policy?**
  confidence: 0
  disposition: deferred
  rationale: captured-while-fresh per [[PL-203]]; design-phase decision — hint: Retroactive design exercise to ground the choice

## Exploration Plan

Treat as a cross-cutting review. After Foundation lands, review each Foundation/Resilience primitive's design against the budget. Then build the missing policy primitives.

## Technical Constraints

**Dependencies (upstream):** None directly

**Dependencies (downstream):** Every other primitive should respect the budget at design — review each Foundation/Resilience primitive's design against this

**ADR §9 boundary:** cross-cutting policy — touches every primitive's design

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
- Cross-cutting review surfaced concrete budget; each primitive's design respects it; missing policy primitives bounded and small.

**NO-GO if:**
- Budget review reveals fundamental scale mismatch — substrate cannot support intended fleet size without redesign of channel storage.

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

**Rationale:** Captured-while-fresh per PL-203; per-primitive design follows operator promotion. T-1991 is precedent.

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
