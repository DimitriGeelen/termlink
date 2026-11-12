---
id: T-3746
name: "X-4: PH-5 circuit sub-design and H-1 / H-27 value check"
description: >
  arc-011 plan unit X-4 (docs/design/interactive-agent-communication-08-plan.md v0.3, approved G-0 2026-11-12): PH-5 circuit sub-design, RC-7..RC-9 pseudo-code (F-43) and the H-1 / H-27 value check

status: captured
workflow_type: inception
owner: agent
horizon: later
tags: [arc:arc-011]
components: []
related_tasks: [T-3357, T-3731, T-3383, T-3735]
plan_unit: X-4
arc_id: arc-011
depends_on: [T-3731]
origin: {kind: "operator", source: "T-3357", ref: "G-0"}
created: 2026-11-12T16:57:29Z
last_update: 2026-11-12T16:57:29Z
date_finished: null
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 5            # int 0..9. T-3357: PH-5 (P-62, P-63), cross-subsystem. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.8                    # float 0..1. T-3357: decides whether PH-5 is built at all (G-6, P-62, P-63). Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
                                  # T-3172: deliberately EMPTY — set a considered value. 0.5 (and 0.3/0.4) score 2, the same as "not assessed".
---

# T-3746: X-4: PH-5 circuit sub-design and H-1 / H-27 value check

## Problem Statement

**Background / trigger:** Exploration **X-4** of the arc-011 step-8 build plan
`docs/design/interactive-agent-communication-08-plan.md` v0.3, approved by the operator 2026-11-12 (G-0, T-3357
`## Decisions`): the PH-5 circuit sub-design, the RC-7..RC-9 pseudo-code (panel finding F-43) and the H-1 / H-27 value
check. Input: T-3383 (30 days of real hub outages, H-1). Asked after P-58.

**Problem / hypothesis:** A direct agent-to-agent circuit keeps agents talking when both hubs are down, but it costs a
new credential type and a second path to secure; whether it is worth building depends on how often hubs are really
down (H-1) and what the circuit would save (H-27).

**Goal:** Put one question to the operator: GO on the circuit; stop at PH-4 and keep the hub path; or re-rule PN-1.
Decide on the H-1 and H-27 numbers.

**Expected outcome:** A recorded decision; on GO, the RC-7..RC-9 pseudo-code and the circuit sub-design that P-62
(T-3735) builds, and gate G-6 can be passed.

Depends on: T-3731 (P-58 built: DM encryption done (FP 6.2.6.c); estate: T-3383 30-day data)
Depends on: gate G-0 (decision: plan approved 2026-11-12; satisfied)
Unblocks: gate G-6 (PH-5 move), and through it P-62 (T-3735).

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

- **IW-1: Is the direct circuit worth building given 30 days of real hub outages and the cost of the hub path: GO, stop at PH-4, or re-rule PN-1?**
  confidence: 1
  disposition: deferred
  rationale: plan 08 v0.3 section 9 X-4; needs the T-3383 outage data and P-58 done

## Exploration Plan

1. Collect the H-1 numbers from T-3383 (30 days of hub outages) and estimate H-27 (value of the circuit) from PH-4 data.
2. Write the RC-7..RC-9 pseudo-code (F-43) and the circuit sub-design in a research artifact `docs/reports/T-3746-*.md`.
3. Put the single question to the operator with the numbers.
Time-box: one session after the data exists.

## Technical Constraints

<!-- What platform, browser, network, or hardware constraints apply?
     For web apps: HTTPS requirements, browser API restrictions, CORS, device support.
     For hardware APIs (mic, camera, GPS, Bluetooth): access requirements, permissions model.
     For infrastructure: network topology, firewall rules, latency bounds.
     Fill this BEFORE building. Discovering constraints after implementation wastes sessions. -->

## Scope Fence

IN: the H-1/H-27 value check, the circuit sub-design and RC-7..RC-9 pseudo-code. OUT: building the circuit (P-62).

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
- H-1 passes (measured hub outages are frequent or long enough that agents lose real work), and
- H-27 shows the circuit's value exceeds the cost of a second secured path, with RC-7..RC-9 specified

**NO-GO if:**
- H-1 fails (hubs are rarely down), so the hub path is enough: stop at PH-4, or the operator re-rules PN-1

## Verification

# Plan X-4 check: the decision is recorded with `fw inception decide T-3746 go|no-go` in the operator's words, citing
# the H-1 and H-27 numbers.

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

**Rationale:** Plan 08 v0.3 section 9: decided on the H-1 and H-27 numbers after P-58; input T-3383 (30 days of outages) not yet in.

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
