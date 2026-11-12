---
id: T-3748
name: "X-6: re-wrap cost for long encrypted histories"
description: >
  arc-011 plan unit X-6 (docs/design/interactive-agent-communication-08-plan.md v0.3, approved G-0 2026-11-12): Re-wrap cost for long histories (R-74, R-80) sized from PH-4 data

status: captured
workflow_type: inception
owner: agent
horizon: later
tags: [arc:arc-011]
components: []
related_tasks: [T-3357, T-3731, T-3738]
plan_unit: X-6
arc_id: arc-011
depends_on: [T-3731]
origin: {kind: "operator", source: "T-3357", ref: "G-0"}
created: 2026-11-12T16:57:31Z
last_update: 2026-11-12T16:57:31Z
date_finished: null
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 3            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.6                    # float 0..1. T-3357: sizes custody and history sharing (P-65). Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
                                  # T-3172: deliberately EMPTY — set a considered value. 0.5 (and 0.3/0.4) score 2, the same as "not assessed".
---

# T-3748: X-6: re-wrap cost for long encrypted histories

## Problem Statement

**Background / trigger:** Exploration **X-6** of the arc-011 step-8 build plan
`docs/design/interactive-agent-communication-08-plan.md` v0.3, approved by the operator 2026-11-12 (G-0, T-3357
`## Decisions`): the re-wrap cost for long histories (R-74, R-80), sized from PH-4 data. Asked after P-58 (T-3731).

**Problem / hypothesis:** When a new reader joins a conversation or project mail, sharing its encrypted history means
re-wrapping message keys; at our history sizes this may be cheap or may be too slow to do on every join.

**Goal:** Put one question to the operator: GO on re-wrapping full histories, or limit history sharing; decided on
measured PH-4 data.

**Expected outcome:** A recorded decision with the measured sizes; it sets how P-65 (T-3738) handles custody and history.

Depends on: T-3731 (P-58 built: PH-4 data)
Depends on: gate G-0 (decision: plan approved 2026-11-12; satisfied)
Unblocks: P-65 (T-3738, decision: sizing decided).

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

- **IW-1: Is re-wrapping the keys of long encrypted histories affordable at our sizes (GO), or should history sharing be limited?**
  confidence: 1
  disposition: deferred
  rationale: plan 08 v0.3 section 9 X-6; needs PH-4 data from P-58

## Exploration Plan

1. From P-58's data, measure history sizes per conversation and project and time a re-wrap at the largest.
2. Record the numbers in a research artifact `docs/reports/T-3748-*.md`.
3. Put the single question to the operator.
Time-box: one session after the data exists.

## Technical Constraints

<!-- What platform, browser, network, or hardware constraints apply?
     For web apps: HTTPS requirements, browser API restrictions, CORS, device support.
     For hardware APIs (mic, camera, GPS, Bluetooth): access requirements, permissions model.
     For infrastructure: network topology, firewall rules, latency bounds.
     Fill this BEFORE building. Discovering constraints after implementation wastes sessions. -->

## Scope Fence

IN: measuring re-wrap cost and the sharing decision. OUT: building custody (P-65) or changing DM encryption (P-58).

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
- A re-wrap of the largest measured history completes within a join's acceptable delay on the hosts we run

**NO-GO if:**
- Re-wrap time or size grows past that bound, so history sharing is limited (e.g. to a window) instead

## Verification

# Plan X-6 check: the decision is recorded with `fw inception decide T-3748 go|no-go` in the operator's words, citing
# the measured sizes.

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

**Rationale:** Plan 08 v0.3 section 9: decided on PH-4 data, which P-58 produces.

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
