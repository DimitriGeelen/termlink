---
id: T-3747
name: "X-5: Track A after the A0 pilot: A1..A3 or not"
description: >
  arc-011 plan unit X-5 (docs/design/interactive-agent-communication-08-plan.md v0.3, approved G-0 2026-11-12): Track A after the A0 pilot: go to A1..A3 or not

status: captured
workflow_type: inception
owner: agent
horizon: later
tags: [arc:arc-011]
components: []
related_tasks: [T-3357, T-3725, T-3733]
plan_unit: X-5
arc_id: arc-011
depends_on: [T-3725]
origin: {kind: "operator", source: "T-3357", ref: "G-0"}
created: 2026-11-12T16:57:30Z
last_update: 2026-11-12T16:57:30Z
date_finished: null
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 5            # int 0..9. T-3357: accounts on every host (P-60, P-66). Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.7                    # float 0..1. T-3357: decides Track A A1..A3 (P-60) and so A4 (P-66). Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
                                  # T-3172: deliberately EMPTY — set a considered value. 0.5 (and 0.3/0.4) score 2, the same as "not assessed".
---

# T-3747: X-5: Track A after the A0 pilot: A1..A3 or not

## Problem Statement

**Background / trigger:** Exploration **X-5** of the arc-011 step-8 build plan
`docs/design/interactive-agent-communication-08-plan.md` v0.3, approved by the operator 2026-11-12 (G-0, T-3357
`## Decisions`): Track A after the A0 pilot, go to A1..A3 or not. Asked after P-52 (T-3725) produces the A0 report.

**Problem / hypothesis:** All agents share one account, so one agent can read another's keys. The A0 pilot runs one
agent under a separate account through the audited helper; its report shows what separate accounts cost and break
(including the managed-settings question 12.6).

**Goal:** Put one question to the operator: give every agent its own account (GO to A1..A3), or not (NO-GO).

**Expected outcome:** A recorded decision on the A0 report; on GO, P-60 (T-3733) can start.

Depends on: T-3725 (P-52 built: A0 pilot report)
Depends on: gate G-0 (decision: plan approved 2026-11-12; satisfied)
Unblocks: P-60 (T-3733, decision: GO).

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

- **IW-1: After the A0 pilot, give every agent its own account (GO to A1..A3) or not?**
  confidence: 1
  disposition: deferred
  rationale: plan 08 v0.3 section 9 X-5; decided on the A0 report that P-52 produces

## Exploration Plan

1. Read the A0 pilot report from P-52 (T-3725): what broke, the managed-settings answer (12.6), the C-15 event trail.
2. Estimate the A1..A3 cost per host and what each stage protects (R-82.e).
3. Put the single GO / NO-GO question to the operator.
Time-box: one session after the A0 report exists.

## Technical Constraints

<!-- What platform, browser, network, or hardware constraints apply?
     For web apps: HTTPS requirements, browser API restrictions, CORS, device support.
     For hardware APIs (mic, camera, GPS, Bluetooth): access requirements, permissions model.
     For infrastructure: network topology, firewall rules, latency bounds.
     Fill this BEFORE building. Discovering constraints after implementation wastes sessions. -->

## Scope Fence

IN: the A1..A3 go/no-go on the A0 evidence. OUT: creating accounts (P-60); the orchestrator without root (P-66).

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
- The A0 pilot ran without manual recovery, the helper refused every request outside its settings, and the managed-settings question (12.6) has a workable answer

**NO-GO if:**
- The A0 pilot shows separate accounts break agent tooling in ways the helper cannot contain, or the per-host cost outweighs the key-isolation gain

## Verification

# Plan X-5 check: the decision is recorded with `fw inception decide T-3747 go|no-go` in the operator's words, citing
# the A0 report.

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

**Rationale:** Plan 08 v0.3 section 9: decided on the A0 pilot report, which P-52 produces in PH-3.

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
