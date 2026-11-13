---
id: T-3744
name: "X-1: R-41 typed work-message slice: design, specs, placement"
description: >
  arc-011 plan unit X-1 (docs/design/interactive-agent-communication-08-plan.md v0.3,
  approved G-0 2026-11-12): R-41 typed work-message slice (OD-13): design, E-n specs
  (05 declares a gap), and placement

status: work-completed
workflow_type: inception
owner: agent
horizon: null
tags: [arc:arc-011]
components: []
related_tasks: [T-3357, T-3728, T-3741]
plan_unit: X-1
arc_id: arc-011
depends_on: [T-3728]
origin: {kind: "operator", source: "T-3357", ref: "G-0"}
created: 2026-11-12T16:57:26Z
last_update: 2026-11-13T17:16:09Z
date_finished: 2026-11-13T17:16:09Z
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 3            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.6                    # float 0..1. T-3357: unblocks one PH-6 unit (P-68) and fixes R-41's spec gap. Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
                                  # T-3172: deliberately EMPTY — set a considered value. 0.5 (and 0.3/0.4) score 2, the same as "not assessed".
bvp_scores_proposed:
  - ts: '2026-11-13T17:16:09Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 3
      D2: 3
      D3: 3
      D4: 3
      F-TRUST: 3
      F-SURFACE: 3
      F-CONV-RECALL: 3
    rationale: D1=3 (no-signal); D2=3 (no-signal); D3=3 (no-signal); D4=3 
      (no-signal); F-TRUST=3 (no-signal); F-SURFACE=3 (no-signal); 
      F-CONV-RECALL=3 (no-signal)
    rubric_sha: e4a00f38e801
    driver_set: product
---

# T-3744: X-1: R-41 typed work-message slice: design, specs, placement

## Problem Statement

**Background / trigger:** Exploration **X-1** of the arc-011 step-8 build plan
`docs/design/interactive-agent-communication-08-plan.md` v0.3, approved by the operator 2026-11-12 (G-0, T-3357
`## Decisions`): the R-41 typed work-message slice (OD-13), its design, its E-n specs (05 declares a gap, CR-8.3) and
its placement (plan section 8). The operator decides GO or NO-GO; nothing is asked before the inputs exist (after P-55).

**Problem / hypothesis:** R-41 is kept as a later slice (OD-13 2c): `assignment.v0` and `result_manifest.v0` with an
expiry, authorisation checked at execution time, a fencing token and the CAND-14 obligation states. Today they are
helpers only; no verb uses them, and EV/COV have no spec for them. Shipping the format alone would put work orders on
the wire without the check that makes them safe to act on (plan 8.3).

**Goal:** Put one question to the operator: build the full slice in PH-6 after grants (A), ship the format alone
earlier (B), or drop R-41 (C). Plan recommendation: **A**.

**Expected outcome:** A recorded decision; on GO, the E-n specs for R-41.e (a)..(e) written so P-68 (T-3741) can be
verified, and the 05a coverage table shows R-41 with a non-empty E-n list.

Depends on: T-3728 (P-55 built: record and grant formats fixed)
Depends on: gate G-0 (decision: plan approved 2026-11-12; satisfied)
Unblocks: P-68 (T-3741, decision: GO).

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

- **IW-1: Build the full R-41 slice in PH-6 after grants (A), the format alone earlier (B), or drop R-41 (C)?**
  confidence: 2
  disposition: deferred
  rationale: plan 08 v0.3 section 9 recommends A (section 8.3); asked only after P-55 fixes the record and grant formats

## Exploration Plan

1. After P-55 (T-3728) closes, re-read REQ R-41 and OD-13, FP 4.2 and 6.2.7.a (4), and the record/grant formats P-55 fixed.
2. Draft the R-41 E-n specs for (a) schema validation with refusal reason, (b) expiry, (c) execution-time authorisation, (d) fencing token, (e) CAND-14 obligation states (CR-8.3), in a research artifact `docs/reports/T-3744-*.md`.
3. Put the single A/B/C question to the operator (one decision, background, options, recommendation).
Time-box: one session.

## Technical Constraints

<!-- What platform, browser, network, or hardware constraints apply?
     For web apps: HTTPS requirements, browser API restrictions, CORS, device support.
     For hardware APIs (mic, camera, GPS, Bluetooth): access requirements, permissions model.
     For infrastructure: network topology, firewall rules, latency bounds.
     Fill this BEFORE building. Discovering constraints after implementation wastes sessions. -->

## Scope Fence

IN: the R-41 design question, the E-n specs for R-41.e, and the placement. OUT: building anything (that is P-68 after
a GO); changing grants (P-56) or epochs and fencing (P-58).

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

<!-- Fill these BEFORE writing the recommendation. The placeholder detector will block review/decide if left empty. -->
**GO if:**
- The R-41.e clauses (a)..(e) each have an E-n spec that P-68 can be verified against
- The full slice can rely on the grants of P-56 and the fencing of P-58, so execution-time authorisation is real

**NO-GO if:**
- No agent workflow needs typed work orders by PH-6 (free text suffices), or
- The slice cannot be made safe without new mechanisms beyond P-56 and P-58

## Verification

# Plan X-1 check: the decision is recorded with `fw inception decide T-3744 go|no-go` in the operator's words; on GO
# the 05a coverage table shows R-41 with a non-empty E-n list.

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).
#
# Toolchain hint (L-291): if a GO decision will mean editing *.vbproj/*.csproj/*.xaml,
# *.go, Cargo.toml, tsconfig.json, or pom.xml in the build task, plan to add the
# matching build command (dotnet build / go build / cargo check / tsc --noEmit /
# mvn compile) to that build task's ## Verification — P-011 only runs what you write.

## Recommendation

**Recommendation:** GO

**Rationale:** Plan 08 v0.3 section 9 recommends A: build the full R-41 slice in PH-6 after grants; a format-only slice would ship work orders without the execution-time check (section 8.3). Decide after P-55.

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

**Rationale**: Recommendation: GO

Rationale: Plan 08 v0.3 section 9 recommends A: build the full R-41 slice in PH-6 after grants; a format-only slice would ship work orders without the execution-time check (section 8.3). Decide after P-55.

**Date**: 2026-11-13T17:16:06Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-11-13T17:16:06Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Recommendation: GO

Rationale: Plan 08 v0.3 section 9 recommends A: build the full R-41 slice in PH-6 after grants; a format-only slice would ship work orders without the execution-time check (section 8.3). Decide after P-55.

### 2026-11-13T17:16:07Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)
- **Reason:** Inception decision in progress

### 2026-11-13T17:16:09Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO

## Reviewer Verdict (v1.5)

- **Scan ID:** R-19cb4ff8
- **Timestamp:** 2026-11-13T17:16:11Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Scope:** static anti-pattern scan; PASS means no anti-pattern was found, not that the criteria hold
- **Needs Human:** no
- **Findings:** none

## Recommendation Verdict (v1.0)

- **Scan ID:** RC-c9962abc
- **Timestamp:** 2026-11-13T17:16:11Z
- **Overall:** UNVERIFIED
- **Claims:** 0
- No verifiable claims found in ## Recommendation
