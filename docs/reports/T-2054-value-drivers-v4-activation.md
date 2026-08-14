# T-2054 — Value drivers v4: F-EVIDENCE + F-CONTAINMENT + F-FEDERATION activation

> **Retrospective consolidation.** Written 2026-08-14 under T-2716 from the
> recorded contents of `.tasks/completed/T-2054-value-drivers-v4-f-evidence--f-containme.md`.
> The exploration and decision below happened on 2026-06-09; this file relocates
> that trail out of the archived task file into `docs/reports/` per C-001. No
> finding here is new.
>
> **Decision on record: GO** (2026-06-09T11:23:12Z), with the operator recording
> it via Watchtower per §ACD on `policy/value-drivers.yaml`.

## The question

`policy/value-drivers.yaml` v3 shipped 2026-06-01 with two active free drivers
(F-RECALL, F-ORCH) and three of five slots open. Three categories of AEF/TermLink
work over the preceding ~30 days had produced **structurally under-rewarded
value** that D1-D4 plus the two active free drivers do not score.

This inception asked whether three new drivers earn those slots, and what their
weights should be, so that future task ranking reflects current focus.

**For whom:** the operator (re-prioritisation transparency); the BVP estimator
(T-1922 heuristic extension); auto-promote eligibility, when re-enabled.

**Why then:** v3 had freshly landed with three slots open and no forced
add-one-drop-one trade-off — activating was structurally cheaper then than later.
The arc-parallel-substrate work (T-2018) was the next focus, and a
TermLink-specific driver (F-FEDERATION) would re-rank substrate work upward by
design.

## The three axes

Each was argued as *new meaning*, not a louder restatement of an existing driver —
the CLAUDE.md activation bar.

| Driver | Axis | Distinct from |
|---|---|---|
| **F-EVIDENCE** | verifiability / falsifiability of claims — AC reform, the fresh re-smoke pattern, T-1731 Human-AC enforcement | D2 mistake-rate |
| **F-CONTAINMENT** | blast-radius bounding — G-058's 19-day silent failure, T-2052's pre-commit blob-size gate, Tier-0, secret-rotation auto-heal | D1 strengthens-from-stress |
| **F-FEDERATION** | cross-hub state coherence in TermLink specifically — the G-060 chat-arc federation gap, DM federation lag, MCP parity work | F-ORCH routable-surface expansion |

## Assumptions

- **A1** — all three pass the activation bar: *"a free driver is only justified
  when the current focus is an axis D1-D4 do not **mean**, not louder."*
- **A2** — three additions exhaust the open slots; further drivers require
  add-one-drop-one and belong to a separate inception.
- **A3** — F-CONTAINMENT is genuinely orthogonal to D1: strengthens-from-stress is
  a *response* property, containment is a *propagation-bound* property.
- **A4** — F-FEDERATION earns its slot specifically for TermLink work, distinct
  from F-ORCH because routable-surface expansion does not mean cross-hub
  consistency.
- **A5** — the §ACD gate fires correctly under `$CLAUDECODE=1`; the operator
  records the decision via Watchtower.

## Open questions and their dispositions

### IW-1 — Does F-CONTAINMENT double-count with D2 Reliability at low bands?

*confidence 2 · answered*

Per the T-2157 §Verdict pattern, bands 0-2 may overlap with adjacent drivers, but
this only matters for human-confirmed scores — the estimator does not score free
drivers in v1. Ship the full 0-5 scale and calibrate after ≥10 confirmed scores,
parallel to the F-RECALL band-0-2 follow-up from T-2157.

### IW-2 — Should F-FEDERATION's rubric reward primitives that *enable* federation, or only mechanisms that *perform* it?

*confidence 3 · answered*

**Both**, via a banded rubric: 0-1 single-hub blind / manual scripting; 2-3
composition / verb-default; 4-5 primitives / autonomous reconciliation. This
mirrors F-ORCH's "score capability uplift" guardrail pattern.

### IW-3 — Is F-LEGIBILITY (observability) overlap with F-RECALL meaningful, or a clean carve?

*confidence 2 · **deferred***

The arc-002 (observability) verdict is needed before activating. The carve
documentation was captured to preserve the structural reasoning. **Activation
gate:** when observability work clearly exceeds the D3 + F-RECALL composite — ≥3
tasks where the human says *"this is mostly observability, not retrievability."*

This is the one question left open, and it is left open deliberately: it is the
fourth slot, and taking it without evidence would defeat the activation bar the
other three had to clear.

### IW-4 — What weights match the current focus signal?

*confidence 2 · answered*

- **F-EVIDENCE = 5** — below F-RECALL (6), at parity with F-ORCH (5);
  verifiability is a structural baseline
- **F-CONTAINMENT = 4** — below F-EVIDENCE; containment bounds consequences and is
  less directional than verifiability
- **F-FEDERATION = 5** — parity with F-ORCH; both are substrate-uplift drivers

The operator can re-weight via `fw bvp weight` post-activation if the focus signal
shifts.

### IW-5 — Does activating three at once destabilise rankings of in-flight tasks?

*confidence 2 · answered*

**No.** Missing `bvp_scores` keys are treated as 0 per the CLAUDE.md normalisation
rule, confirmed in T-2157. In-flight tasks ranked before v4 retain their D1-D4 +
F-RECALL + F-ORCH scores untouched, and re-scoring is opt-in per task via
`fw bvp confirm`.

## Decision — GO

Activate all three drivers, using the three open slots cleanly, at the weights
above, with the F-LEGIBILITY carve documented and gated on the arc-002 verdict.
