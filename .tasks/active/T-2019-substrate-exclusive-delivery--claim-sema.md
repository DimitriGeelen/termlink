---
id: T-2019
name: "Substrate: exclusive-delivery / claim semantics for work topics"
description: >
  §6 primitive 1 (Foundation, hard-dep). Topics are broadcast: two subscribers on
  a work topic both see every post, so a queue-of-tasks model has two agents grab
  the same unit. No lock/lease/CAS/claim verb anywhere. Foundation for any safe handoff
  to exactly one consumer; also why the orchestrator cannot be a passive pull-queue.

status: captured
workflow_type: inception
owner: human
horizon: later
tags: [arc:arc-parallel-substrate]
components: []
related_tasks: [T-2018]
created: 2026-06-07T11:35:37Z
last_update: '2026-06-07T11:41:29Z'
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
  - ts: '2026-06-07T11:41:29Z'
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

# T-2019: Substrate: exclusive-delivery / claim semantics for work topics

## Problem Statement

§6 primitive of [arc-parallel-substrate](../../docs/architecture/parallel-execution-substrate.md). **Group:** Foundation. **§9 boundary:** hard-dep per §9.

**Role per ADR §6:** Topics are broadcast: two subscribers on a 'work' topic both see every post. There is no lock / lease / CAS / claim verb anywhere in the codebase. Without it, no work topic can hand a unit to exactly one consumer — the naive queue-of-tasks design collides on first use.

**Why captured now:** Without claim semantics no other Foundation primitive (T-2020, T-2021) has anything to coordinate over. Build order's leading edge.

**Status disclosure ([[PL-203]]):** Filed `status: captured, horizon: later`. BVP scores will be estimator-proposed (not confirmed) until promotion. No commitment to RPC shape, build order, or cost-estimate is locked at filing. Per-primitive design phase opens at operator promotion via `fw bvp confirm` + `--horizon now`.

## Assumptions

<!-- Key assumptions to test. Register with: fw assumption add "Statement" --task T-XXX -->

## Open Questions

- **IW-1: Semantics — hold-claim-until-ack, lease-with-renewal, or CAS-on-offset?**
  confidence: 0
  disposition: deferred
  rationale: captured-while-fresh per [[PL-203]]; design-phase decision — hint: Each has different failure modes; ack-based is simplest but blocks if worker dies

- **IW-2: Storage — in-memory (lost on hub restart) or persisted to SQLite (durable)?**
  confidence: 0
  disposition: deferred
  rationale: captured-while-fresh per [[PL-203]]; design-phase decision — hint: Resilience layer (T-2025) covers persistence broadly; may share infra

- **IW-3: Interaction with persistent cursors — does claim advance the cursor, or is it orthogonal?**
  confidence: 0
  disposition: deferred
  rationale: captured-while-fresh per [[PL-203]]; design-phase decision — hint: Existing cursor semantics are subscriber-scoped; claim is sender-scoped

- **IW-4: Is `claim` a new RPC verb or a flag on `subscribe`?**
  confidence: 0
  disposition: deferred
  rationale: captured-while-fresh per [[PL-203]]; design-phase decision — hint: Affects backward compat and how AEF orchestrator builds against it

## Exploration Plan

At promotion time: (1) prototype each semantic option as a 1-day spike; (2) measure failure-mode behavior under simulated worker-crash; (3) decide on one shape; (4) file the build task with locked design. Bake decisions into ADR §6.1 update.

## Technical Constraints

**Dependencies (upstream):** None (Foundation primitive)

**Dependencies (downstream):** T-2020 (registry), T-2021 (pull/assign), T-2026 (typed agent-launch — for ledger-write serialization)

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
- Single shape chosen with measured failure-mode behavior; spike code points at a 1-2 day build; design backward-compatible with existing subscribers.

**NO-GO if:**
- Novel-mechanism surfaces (e.g. distributed-consensus-style protocol); spike reveals failure modes unbounded; build scope >1 week.

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

**Rationale:** Captured-while-fresh per PL-203 + arc-parallel-substrate. Per-primitive design/build deferred until operator promotes via fw bvp confirm + horizon=now. No commitment to RPC shape made at filing.

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
