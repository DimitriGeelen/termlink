---
id: T-2054
name: "value-drivers v4: F-EVIDENCE + F-CONTAINMENT + F-FEDERATION activation"
description: >
  Inception: value-drivers v4: F-EVIDENCE + F-CONTAINMENT + F-FEDERATION activation

status: work-completed
workflow_type: inception
owner: human
horizon: null
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-06-08T12:18:48Z
last_update: 2026-06-09T11:23:12Z
date_finished: 2026-06-09T11:23:12Z
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 3            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.5                    # float 0..1. Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
---

# T-2054: value-drivers v4: F-EVIDENCE + F-CONTAINMENT + F-FEDERATION activation

## Problem Statement

<!-- What problem are we exploring? For whom? Why now? -->

`policy/value-drivers.yaml` v3 shipped 2026-06-01 with 2 active free drivers (F-RECALL, F-ORCH) and 3 of 5 slots open. Three categories of AEF/TermLink work over the last ~30 days produced structurally under-rewarded value that D1-D4 + F-RECALL + F-ORCH do not score. This inception evaluates whether three new drivers earn slots and what their weights should be, so future task ranking reflects current focus.

**For whom:** the operator (re-prioritisation transparency); the BVP estimator (T-1922 heuristic extension); auto-promote eligibility (when re-enabled).

**Why now:** v3 freshly landed, 3 slots open without forced add-one-drop-one trade-off. Activating now is structurally cheaper than later. The arc-parallel-substrate work (T-2018) is the next focus and a TermLink-specific driver (F-FEDERATION) would re-rank substrate work upward as designed.

## Assumptions

A1: All three proposed drivers pass the CLAUDE.md activation bar — "a free driver is only justified when the current focus is an axis D1-D4 do not *mean*, not louder."
A2: Three additions exhaust the open slots; further drivers require add-one-drop-one and should be filed under a separate inception when justified.
A3: F-CONTAINMENT is genuinely orthogonal to D1 (strengthens-from-stress is a *response* property; containment is a *propagation-bound* property).
A4: F-FEDERATION earns its slot specifically for TermLink work — distinct from F-ORCH because routable-surface expansion does not mean cross-hub consistency.
A5: The §ACD gate fires correctly under $CLAUDECODE=1 — operator records decision via Watchtower.

## Open Questions

- **IW-1: Does F-CONTAINMENT double-count with D2 Reliability at low bands?**
  confidence: 2
  disposition: answered
  rationale: T-2157 §Verdict pattern — bands 0-2 may overlap with adjacent drivers, but only matters for human-confirmed scores (estimator doesn't score free drivers in v1). Ship full 0-5 scale; calibrate after ≥10 confirmed scores (parallel to F-RECALL band-0-2 follow-up T-NEW-B from T-2157).

- **IW-2: Should F-FEDERATION's rubric reward primitives that enable federation, or only mechanisms that perform it?**
  confidence: 3
  disposition: answered
  rationale: Both — rubric structured as 0-1 (single-hub blind / manual scripting), 2-3 (composition / verb-default), 4-5 (primitives / autonomous reconciliation). Mirrors F-ORCH's "score capability uplift" guardrail pattern.

- **IW-3: Is F-LEGIBILITY (observability) overlap with F-RECALL meaningful, or is it a clean carve?**
  confidence: 2
  disposition: deferred
  rationale: arc-002 (observability) verdict needed before activating; carve documentation captured to preserve structural reasoning. Activation gate: when observability work clearly exceeds D3 + F-RECALL composite (≥3 tasks where the human says "this is mostly observability, not retrievability").

- **IW-4: What weights match the current focus signal?**
  confidence: 2
  disposition: answered
  rationale: Proposed F-EVIDENCE=5 (below F-RECALL=6, parity with F-ORCH=5 — verifiability is structural baseline), F-CONTAINMENT=4 (below F-EVIDENCE; containment is consequence-bounding, less directional than verifiability), F-FEDERATION=5 (parity with F-ORCH — both are substrate-uplift drivers). Operator can re-weight via `fw bvp weight` post-activation if focus signal shifts.

- **IW-5: Does activating three at once destabilise rankings of in-flight tasks?**
  confidence: 2
  disposition: answered
  rationale: No — bvp_scores keys missing → treated as 0 (per CLAUDE.md normalisation rule, confirmed in T-2157). In-flight tasks ranked before v4 retain their D1-D4 + F-RECALL + F-ORCH scores untouched. Re-scoring is opt-in via `fw bvp confirm` per task.

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

## Exploration Plan

This inception is a desk-research evaluation — no spikes needed. Evidence is the recent commit log + gap register + arc work, all already in repo:

1. **Apply "new meaning" test to each candidate** — cite which D1-D4 / F-RECALL / F-ORCH the driver would double-count with, and explain how the proposed driver is orthogonal. (Done in this document.)
2. **Cite current-focus evidence per candidate** — point at specific tasks, gaps, or PLs that show the axis is currently being worked on but under-rewarded.
3. **Propose rubric (0-5) per candidate** — mirror F-RECALL/F-ORCH structure: low bands = absent or manual, high bands = primitives / autonomous mechanisms.
4. **Propose weight per candidate** — relative to existing drivers, with guardrails against ranking dominance.
5. **Identify the carve** — F-LEGIBILITY documented as not-yet-active so the structural reasoning isn't lost.

Time-box: this task itself is the artifact (no separate spike). Decision via Watchtower.

## Technical Constraints

- **§ACD gate fires under $CLAUDECODE=1** on weight/driver edits to `policy/value-drivers.yaml` (T-1932). Operator records decision via Watchtower; agent cannot directly edit on GO.
- **Build slices on GO** must be small and reversible per CLAUDE.md "no one-way doors": yaml edits + estimator extension (T-NEW-A from T-2157 follow-up applies) + rubric calibration (defer to T-NEW-B-style data-driven pass once ≥10 human-confirmed scores accumulate per driver).
- **Cap enforcement:** activating three uses all open slots (2 active + 3 new = 5 active = cap). Subsequent drivers require add-one-drop-one and a fresh inception.

## Scope Fence

**IN scope:**
- Evaluating F-EVIDENCE, F-CONTAINMENT, F-FEDERATION against the activation bar
- Proposing rubric + weight + retire_when per driver
- Carving F-LEGIBILITY (documentation only)
- Recommending GO with operator-decides-via-Watchtower

**OUT of scope:**
- Touching `policy/value-drivers.yaml` directly (§ACD gate — operator's call)
- Modifying the BVP estimator heuristics (separate build task on GO)
- Activating F-LEGIBILITY or F-AUTONOMY (carved, awaiting arc verdicts)
- Re-scoring existing tasks with new drivers (opt-in via `fw bvp confirm`)
- Rubric calibration of bands 0-2 (separate data-driven pass once corpus accumulates)

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
- All three proposed drivers pass the "new meaning, not louder D1-D4" bar (operator agrees with the orthogonality argument)
- Activating now (3 open slots, no add-one-drop-one) is structurally cheaper than waiting
- F-FEDERATION specifically matches the TermLink-focus directive (re-ranking arc-parallel-substrate work upward)

**NO-GO if:**
- Operator finds any of the three drivers double-counts with an existing driver
- Operator prefers to activate fewer (e.g. only F-EVIDENCE + F-FEDERATION) — file as scope-reduced GO via Watchtower note
- Operator wants to first land arc-002 verdict on observability before fixing the free-driver landscape

**DEFER if:**
- Calibration data on F-RECALL bands 0-2 (still ≤10 human-confirmed scores per T-2157 §Follow-up) suggests rubric design needs more evidence before adding three more rubrics

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

**Recommendation:** GO

**Rationale:**

BVP v3 has 3 open free-driver slots (cap 5, 2 active F-RECALL+F-ORCH). Current AEF work is producing structurally under-rewarded value in three orthogonal axes: (1) verifiability/falsifiability of claims (AC reform, fresh re-smoke pattern, T-1731 Human-AC enforcement — distinct from D2 mistake-rate), (2) blast-radius bounding (G-058 19-day silent failure, T-2052 pre-commit blob-size gate, Tier-0 system, secret rotation auto-heal — distinct from D1 strengthens-from-stress), and (3) cross-hub state coherence in TermLink specifically (G-060 chat-arc federation gap, DM federation lag, MCP parity work — distinct from F-ORCH routable-surface expansion). All three pass the CLAUDE.md activation bar ('new meaning, not louder D1-D4'). Activating all three uses the 3 open slots cleanly; F-LEGIBILITY observability is carved pending arc-002 verdict. Recommendation: GO on v4 with the three drivers activated and the carve documented; weights proposed F-EVIDENCE=5, F-CONTAINMENT=4, F-FEDERATION=5. Operator decides via Watchtower per §ACD on policy/value-drivers.yaml.

**Evidence:**

<!-- Add evidence bullets as exploration progresses (file paths,
     commit hashes, test results). The filing-time recommendation
     can be revised before fw inception decide. -->

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

Rationale:

BVP v3 has 3 open free-driver slots (cap 5, 2 active F-RECALL+F-ORCH). Current AEF work is producing structurally under-rewarded value in three orthogonal axes: (1) verifiability/falsifiability of claims (AC reform, fresh re-smoke pattern, T-1731 Human-AC enforcement — distinct from D2 mistake-rate), (2) blast-radius bounding (G-058 19-day silent failure, T-2052 pre-commit blob-size gate, Tier-0 system, secret rotation auto-heal — distinct from D1 strengthens-from-stress), and (3) cross-hub state coherence in TermLink specifically (G-060 chat-arc federation gap, DM federation lag, MCP parity work — distinct from F-ORCH routable-surface expansion). All three pass the CLAUDE.md activation bar ('new meaning, not louder D1-D4'). Activating all three uses the 3 open slots cleanly; F-LEGIBILITY observability is carved pending arc-002 verdict. Recommendation: GO on v4 with the three drivers activated and the carve documented; weights proposed F-EVIDENCE=5, F-CONTAINMENT=4, F-FEDERATION=5. Operator decides via Watchtower per §ACD on policy/value-drivers.yaml.

Evidence:

**Date**: 2026-06-09T11:23:12Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-06-08T12:20:30Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-06-09T11:23:12Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Recommendation: GO

Rationale:

BVP v3 has 3 open free-driver slots (cap 5, 2 active F-RECALL+F-ORCH). Current AEF work is producing structurally under-rewarded value in three orthogonal axes: (1) verifiability/falsifiability of claims (AC reform, fresh re-smoke pattern, T-1731 Human-AC enforcement — distinct from D2 mistake-rate), (2) blast-radius bounding (G-058 19-day silent failure, T-2052 pre-commit blob-size gate, Tier-0 system, secret rotation auto-heal — distinct from D1 strengthens-from-stress), and (3) cross-hub state coherence in TermLink specifically (G-060 chat-arc federation gap, DM federation lag, MCP parity work — distinct from F-ORCH routable-surface expansion). All three pass the CLAUDE.md activation bar ('new meaning, not louder D1-D4'). Activating all three uses the 3 open slots cleanly; F-LEGIBILITY observability is carved pending arc-002 verdict. Recommendation: GO on v4 with the three drivers activated and the carve documented; weights proposed F-EVIDENCE=5, F-CONTAINMENT=4, F-FEDERATION=5. Operator decides via Watchtower per §ACD on policy/value-drivers.yaml.

Evidence:

### 2026-06-09T11:23:12Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
