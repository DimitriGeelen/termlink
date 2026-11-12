---
id: T-3745
name: "X-2: PH-3 detailed sub-design (04b)"
description: >
  arc-011 plan unit X-2 (docs/design/interactive-agent-communication-08-plan.md v0.3, approved G-0 2026-11-12): PH-3 detailed sub-design (04b): roster, cards, enrolment, Tier 0 interface

status: captured
workflow_type: inception
owner: agent
horizon: later
tags: [arc:arc-011]
components: []
related_tasks: [T-3357, T-3455, T-3423, T-2470, T-3722, T-3723]
plan_unit: X-2
arc_id: arc-011
depends_on: []
origin: {kind: "operator", source: "T-3357", ref: "G-0"}
created: 2026-11-12T16:57:27Z
last_update: 2026-11-12T16:57:27Z
date_finished: null
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 7            # int 0..9. T-3357: all PH-3 units (P-48..P-54). Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.9                    # float 0..1. T-3357: gates all of PH-3 (P-48, P-49 and the units after them). Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
                                  # T-3172: deliberately EMPTY — set a considered value. 0.5 (and 0.3/0.4) score 2, the same as "not assessed".
---

# T-3745: X-2: PH-3 detailed sub-design (04b)

## Problem Statement

**Background / trigger:** Exploration **X-2** of the arc-011 step-8 build plan
`docs/design/interactive-agent-communication-08-plan.md` v0.3, approved by the operator 2026-11-12 (G-0, T-3357
`## Decisions`): the PH-3 detailed sub-design (04b): roster, cards, enrolment, Tier 0 interface. Inputs: T-3455 (PH-3
input) and T-3423 (per-peer least-privilege credentials). Asked at the PH-3 move (gate G-4), after the T-2470 charter
is decided.

**Problem / hypothesis:** ARC 23.3 leaves the PH-3 detail open behind a gate: how hubs admit each other, sign cards and
enrol projects, and how the Tier 0 interface works. The PH-3 units (P-48..P-54) are coarse until this sub-design exists
(PL-6) and may be split by it.

**Goal:** A PH-3 sub-design (04b) good enough to build, and the operator's GO / NO-GO / revise on it.

**Expected outcome:** A recorded decision; on GO, P-48 (T-3722) and P-49 (T-3723) can start, and the plan is re-issued
if the sub-design splits units (ids P-48..P-71 stay, new ones are appended).

Depends on: gate G-4, an operator decision point with no task (decision: PH-3 move; T-2470 charter decided as part of it)
Depends on: gate G-0 (decision: plan approved 2026-11-12; satisfied)
Unblocks: P-48 (T-3722), P-49 (T-3723).

## Assumptions

<!-- Key assumptions to test. Register with: fw assumption add "Statement" --task T-XXX -->

## Open Questions

<!-- T-2190 (T-2186 Slice 4): every IW-N question must be disposed before
     --status work-completed. Disposition gate (agents/task-create/update-task.sh
     check_disposition_gate) refuses on under-disposed inceptions.

     Per-question shape:

       - **IW-1: <question text>**
         confidence: 0-3      (your confidence in your current answer; 0=guess, 3=verified)
         disposition: answered | deferred | dissolved
         rationale: <one-line evidence — file:line, decision id, dialogue ref>

     Never bare yes/no — the gate refuses bare checkboxes. See 050-Inceptions.md
     §Disposition Gate. Bypass: --skip-disposition-gate "rationale" (direct) or
     FW_SKIP_DISPOSITION_GATE=1 (env-var, T-1890 producer/consumer parity).
-->

- **IW-1: Is the PH-3 sub-design (roster, cards, enrolment, Tier 0 interface) good enough to build: GO, NO-GO or revise?**
  confidence: 1
  disposition: deferred
  rationale: plan 08 v0.3 section 9 X-2; asked at G-4 after T-2470, the sub-design does not exist yet

## Exploration Plan

1. At G-4, after the T-2470 charter decision, read ARC 23.3, T-3455 and T-3423.
2. Write the 04b sub-design (roster `termlink.roster/1`, signed cards with seq and TTL, enrolment, Tier 0 interface) as a design document, through the role chain's review.
3. Put the single GO / NO-GO / revise question to the operator.
Time-box: per the role chain's design step.

## Technical Constraints

<!-- What platform, browser, network, or hardware constraints apply?
     For web apps: HTTPS requirements, browser API restrictions, CORS, device support.
     For hardware APIs (mic, camera, GPS, Bluetooth): access requirements, permissions model.
     For infrastructure: network topology, firewall rules, latency bounds.
     Fill this BEFORE building. Discovering constraints after implementation wastes sessions. -->

## Scope Fence

IN: the PH-3 sub-design and the GO decision on it. OUT: building PH-3 units; the T-2470 charter decision itself (gate G-4).

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

<!-- Fill these BEFORE writing the recommendation. The placeholder detector will block review/decide if left empty. -->
**GO if:**
- The sub-design specifies roster, cards, enrolment and the Tier 0 interface well enough that P-48 and P-49 have testable acceptance (FP 6.2.4.c (1), (2))
- No PH-3 unit needs a mechanism the sub-design leaves open

**NO-GO if:**
- The T-2470 charter rules multi-hub federation out of scope, or
- The sub-design cannot keep signed state verifiable without a new trust root

## Verification

# Plan X-2 check: the decision is recorded with `fw inception decide T-3745 go|no-go` in the operator's words.

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

**Rationale:** Plan 08 v0.3 section 9: decided on the sub-design at G-4, after T-2470; inputs T-3455, T-3423 not yet ready.

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
