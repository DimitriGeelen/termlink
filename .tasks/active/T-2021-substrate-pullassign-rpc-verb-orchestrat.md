---
id: T-2021
name: "Substrate: pull/assign RPC verb (orchestrator-to-worker handoff)"
description: >
  §6 primitive 3 (Foundation, hard-dep). Every existing path is push (sender picks
  recipient). No give-me-the-next-unit RPC and no clean inverse for the orchestrator
  to hand a specific unit to a specific worker as a first-class operation.

status: captured
workflow_type: inception
owner: human
horizon: later
tags: [arc:arc-parallel-substrate]
components: []
related_tasks: [T-2018]
created: 2026-06-07T11:36:24Z
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

# T-2021: Substrate: pull/assign RPC verb (orchestrator-to-worker handoff)

## Problem Statement

§6 primitive of [arc-parallel-substrate](../../docs/architecture/parallel-execution-substrate.md). **Group:** Foundation. **§9 boundary:** hard-dep per §9.

**Role per ADR §6:** Every existing TermLink path is push (sender picks recipient). There is no 'give me the next unit' RPC and no clean inverse for the orchestrator to hand a specific unit to a specific worker as a first-class operation.

**Why captured now:** Build order tail of Foundation. Fully informed by T-2019 + T-2020 once they land.

**Status disclosure ([[PL-203]]):** Filed `status: captured, horizon: later`. BVP scores will be estimator-proposed (not confirmed) until promotion. No commitment to RPC shape, build order, or cost-estimate is locked at filing. Per-primitive design phase opens at operator promotion via `fw bvp confirm` + `--horizon now`.

## Assumptions

<!-- Key assumptions to test. Register with: fw assumption add "Statement" --task T-XXX -->

## Open Questions

- **IW-1: Push (orchestrator → worker) vs pull (worker requests next) vs both?**
  confidence: 0
  disposition: deferred
  rationale: captured-while-fresh per [[PL-203]]; design-phase decision — hint: Push fits §6 'orchestrator's explicit act'; pull is friendlier to autonomous workers

- **IW-2: Worker selection policy — round-robin, least-loaded, capability-match — hub-side or orchestrator-side?**
  confidence: 0
  disposition: deferred
  rationale: captured-while-fresh per [[PL-203]]; design-phase decision — hint: Hub-side simplifies clients but couples policy to substrate

- **IW-3: Failure mode — assignment unacked within N seconds → reclaim and reassign?**
  confidence: 0
  disposition: deferred
  rationale: captured-while-fresh per [[PL-203]]; design-phase decision — hint: Ties to T-2019's lease/ack semantics

- **IW-4: Is this a new RPC or a composition of (subscribe + claim + ack)?**
  confidence: 0
  disposition: deferred
  rationale: captured-while-fresh per [[PL-203]]; design-phase decision — hint: Resolves naturally once 1+2 land

## Exploration Plan

After T-2019 + T-2020 land. Likely starts as a composition spike: does (claim + ack) already give us pull/assign for free? If yes, no new RPC; if no, design the minimum new verb.

## Technical Constraints

**Dependencies (upstream):** T-2019 (claim makes assignment exclusive), T-2020 (registry says who's idle)

**Dependencies (downstream):** AEF orchestrator dispatcher — without this verb, the orchestrator pattern is undefined

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
- Composition decision is final; AEF orchestrator can build against the resulting interface; failure mode is named and bounded.

**NO-GO if:**
- Composition route reveals a substrate gap that needs a separate primitive; interface is unstable.

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

**Rationale:** Captured-while-fresh per PL-203; depends on shape of primitives 1+2.

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
