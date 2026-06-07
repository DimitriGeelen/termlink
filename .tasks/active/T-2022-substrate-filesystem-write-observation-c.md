---
id: T-2022
name: "Substrate: filesystem-write observation (CSMA/CD-style collision detection)"
description: >
  §6 primitive 4 (Keystone, SOFT per §9). Nothing watches agent file I/O at runtime.
  This is what would let the system PHYSICALLY detect collisions and therefore makes
  a future optimistic mode safe per §4. novel_mechanism: yes, biggest single build,
  expected to split into sub-pieces. Absence FORCES conservative launch policy; presence
  is precondition (not trigger) for optimistic.

status: captured
workflow_type: inception
owner: human
horizon: later
tags: [arc:arc-parallel-substrate, novel-mechanism]
components: []
related_tasks: [T-2018]
created: 2026-06-07T11:36:29Z
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

# T-2022: Substrate: filesystem-write observation (CSMA/CD-style collision detection)

## Problem Statement

§6 primitive of [arc-parallel-substrate](../../docs/architecture/parallel-execution-substrate.md). **Group:** Keystone. **§9 boundary:** **SOFT per §9** — co-discovered with AEF layer, NOT pre-contracted. The only ADR primitive that is genuinely two-sided unknown..

**Role per ADR §6:** An exhaustive code search found no inotify / fanotify / fs-watch / equivalent at runtime. The hub cannot see what files an agent touches. This absence is what FORCES the conservative collision policy in §4. Its presence is the *precondition* (not the trigger) for an optimistic mode.

**Why captured now:** Capture-while-fresh per [[PL-203]]. Likely splits into sub-tasks at design phase — this inception is primarily an unblocker for an Inception sub-arc.

**Status disclosure ([[PL-203]]):** Filed `status: captured, horizon: later`. BVP scores will be estimator-proposed (not confirmed) until promotion. No commitment to RPC shape, build order, or cost-estimate is locked at filing. Per-primitive design phase opens at operator promotion via `fw bvp confirm` + `--horizon now`.

## Assumptions

<!-- Key assumptions to test. Register with: fw assumption add "Statement" --task T-XXX -->

## Open Questions

- **IW-1: Mechanism — inotify, fanotify, ptrace, LD_PRELOAD wrapper, eBPF, FUSE?**
  confidence: 0
  disposition: deferred
  rationale: captured-while-fresh per [[PL-203]]; design-phase decision — hint: Each has different blind spots and host-portability

- **IW-2: Per-ring20-host viability — does the chosen mechanism work on every host? Container restrictions?**
  confidence: 0
  disposition: deferred
  rationale: captured-while-fresh per [[PL-203]]; design-phase decision — hint: Critical — homogeneous mechanism beats per-host special-casing

- **IW-3: Blind spots — what file ops are NOT observable?**
  confidence: 0
  disposition: deferred
  rationale: captured-while-fresh per [[PL-203]]; design-phase decision — hint: Determines whether the mechanism is sound enough to bear the conservative→optimistic flip

- **IW-4: Cost — per-syscall overhead, kernel buffer pressure, scaling with concurrent agents?**
  confidence: 0
  disposition: deferred
  rationale: captured-while-fresh per [[PL-203]]; design-phase decision — hint: Performance budget vs surfaces-collision-quickly tradeoff

- **IW-5: Granularity — directory-level, file-level, byte-range — and what does AEF layer need?**
  confidence: 0
  disposition: deferred
  rationale: captured-while-fresh per [[PL-203]]; design-phase decision — hint: Co-discovered per §9

## Exploration Plan

**Expected to split.** First step at promotion time: 1-week spike comparing inotify vs fanotify vs LD_PRELOAD on ring20 hosts under simulated agent load. Likely produces a sub-arc rather than a single build task.

## Technical Constraints

**Dependencies (upstream):** None (independent mechanism question)

**Dependencies (downstream):** **THIS IS THE GATE ON THE CONSERVATIVE→OPTIMISTIC FLIP** (§4). Without write-observation, optimistic mode is unsafe. Shipping this does NOT flip policy — that's a separate operator-gated decision against criteria defined in advance (§4 two-step that must not be collapsed).

**ADR §9 boundary:** **SOFT per §9** — co-discovered with AEF layer, NOT pre-contracted. The only ADR primitive that is genuinely two-sided unknown.

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
- Mechanism chosen with measured blind-spot list and per-host viability confirmed; sub-task decomposition produced.

**NO-GO if:**
- All candidate mechanisms have blind spots that invalidate them for the conservative→optimistic gate (in which case the substrate ships nothing and policy stays conservative forever — that's a valid outcome per §4).

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

**Rationale:** Captured-while-fresh per PL-203. SOFT dependency per §9 — co-discovered with AEF layer, not pre-contracted. Likely splits into sub-tasks at design phase.

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
