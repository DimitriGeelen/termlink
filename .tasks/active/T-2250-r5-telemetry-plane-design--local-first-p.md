---
id: T-2250
name: "R5 telemetry plane design — local-first per-agent failure telemetry"
description: >
  Inception: R5 telemetry plane design — local-first per-agent failure telemetry

status: captured
workflow_type: inception
owner: human
horizon: later
arc_id: arc-substrate-fitness
tags: [arc:arc-substrate-fitness]
components: []
related_tasks: [T-2242, T-2243, T-2245]
created: 2026-06-23T07:53:59Z
last_update: 2026-07-02T15:40:51Z
date_finished: null
revisit_at: 2026-07-25          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
revisit_evidence_needed: "R7 hygiene cleanup landed (measurement surface de-noised) + R4 daily-aggregated-push transport validated live"  # T-1451
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 3            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.5                    # float 0..1. Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
---

# T-2250: R5 telemetry plane design — local-first per-agent failure telemetry

## Problem Statement

The arc-002 discovery (F-INSTRUMENTATION) found the substrate is **blind to its
own failure modes**: silent poison-drops (now dead-lettered by R4/T-2243), but
also flaps, breaker trips, reconnect storms, RTT degradation, and the
"can't-self-report" class (an agent that dies cannot post its own death). R5 is
the design node for a **durable, local-first per-agent telemetry plane** that
captures these signals so the next discovery measures from data, not anecdote.
Direction is already human-set by Q2 (local-first capture+aggregate; daily
AGGREGATED push riding R4's durable queue; tiered retention). This inception
turns that direction into a decision-depth design + a GO/NO-GO recommendation.
Full artifact: `docs/reports/T-2250-r5-telemetry-plane-design.md`.

## Assumptions

- Telemetry is a **time-series** data class (retain-aggregated), the OPPOSITE of
  presence (current-state, compact/expire) — the F1 bug was applying forever-raw
  to current-state data (plan §0.5 Q2 insight).
- Local-first capture survives hub/agent loss and sees crashes (no observer
  effect); a centralized collector is needed for actionability.
- R4's durable offline queue (T-2243) is the right transport for the daily push
  (best-effort, dead-lettered on poison).

## Open Questions

- **IW-1: Transport — does the daily push go OVER TermLink or stay fully
  out-of-band?** The plan has an internal tension: R5's one-liner (§4) says
  "NOT-over-TermLink" while Q2's resolution (§0.5) says "daily AGGREGATED push
  OVER TermLink riding R4's queue."
  confidence: 2
  disposition: deferred
  rationale: Recommend Q2's resolution wins (over-TermLink aggregated push; raw stays local) — it's the later, explicit human decision. Surfaced for human confirmation. Report §5.
- **IW-2: What signals does the local recorder capture (the schema)?**
  confidence: 2
  disposition: deferred
  rationale: Proposed set in report §3 (discards, flaps, breaker trips, reconnects, RTT samples, clean-exit marker). Human ratifies scope on GO.
- **IW-3: Sequence — open the R5 build now, or after R7 hygiene de-noises the
  measurement surface?**
  confidence: 2
  disposition: deferred
  rationale: Recommend after R7 (test-topic/audit noise would pollute baseline aggregates). Report §6. Human decides ordering.

## Exploration Plan

Design-only inception (no spikes): (1) classify telemetry vs presence data
classes; (2) propose the captured-signal schema; (3) specify local-first storage
+ retention (tiered rollup, dogfooding Q1); (4) reconcile the transport tension
(IW-1); (5) name the collector shape; (6) surface GO/NO-GO + sequencing. Output
is the research artifact + this task's recommendation. Time-box: this session,
bounded.

## Technical Constraints

- Local-first store must be durable across agent crash + hub loss (rules out an
  in-hub-only record); must not create an observer effect on the hot path.
- The aggregated push must be best-effort and ride R4's queue (no new
  silent-drop path — dogfoods the dead-letter).
- The collection topic must itself be bounded/aggregated (dogfoods Q1 retention —
  do not reintroduce the F1 forever-raw bug on the telemetry topic).

## Scope Fence

**IN:** the telemetry-plane design (signals, storage, retention, transport,
collector) at inception depth; a GO/NO-GO recommendation; sequencing advice.
**OUT:** any implementation (no recorder, no collector, no topic created); the
GO/NO-GO decision itself (Sovereign — human via `fw task review T-2250`); R7
live-host hygiene.

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
- The human ratifies the captured-signal schema (IW-2) and the over-TermLink
  aggregated-push transport (IW-1, Q2 resolution), and wants the telemetry-plane
  build opened (ideally sequenced after R7 per IW-3). The build is then
  decomposed into sized build tasks (recorder / collector / retention), not built
  under this inception id.

**NO-GO if:**
- The R4 dead-letter (T-2243) + existing governor/queue observability are judged
  sufficient for the next discovery, and a full per-agent telemetry plane is more
  cost than the remaining observability gap warrants. (DEFER is the agent's lean:
  design is ready, but opening the build should wait on R7 + the human's call on
  transport.)

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

**Rationale:**

R5 is the arc-002 observable-governance design node (Lock 2, after R4 shipped). Q2 already set direction (local-first capture+aggregate; daily AGGREGATED push riding R4's durable queue; tiered retention). This inception fleshes the design at decision-depth and reconciles an internal plan tension (R5 one-liner says 'NOT-over-TermLink' while Q2 resolved to a daily aggregated push OVER TermLink). DEFER pending the design write-up + the human's GO/NO-GO on transport and on whether to open the build now vs after R7 hygiene de-noises the measurement surface.

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

**Decision**: DEFER

<!-- CORRECTED 2026-06-25: was erroneously recorded NO-GO via Watchtower (misclick);
     the recommendation was DEFER and the recorded rationale itself read as a DEFER
     rationale. Reopened to started-work; awaiting formal re-record via
     `fw inception decide T-2250 defer`. revisit_at + revisit_evidence_needed set
     per T-1451/G-053. -->

**Rationale**: DEFER (per the recommendation below).

Rationale:

R5 is the arc-002 observable-governance design node (Lock 2, after R4 shipped). Q2 already set direction (local-first capture+aggregate; daily AGGREGATED push riding R4's durable queue; tiered retention). This inception fleshes the design at decision-depth and reconciles an internal plan tension (R5 one-liner says 'NOT-over-TermLink' while Q2 resolved to a daily aggregated push OVER TermLink). DEFER pending the design write-up + the human's GO/NO-GO on transport and on whether to open the build now vs after R7 hygiene de-noises the measurement surface.

Evidence:

**Date**: 2026-06-25T06:31:02Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-06-23T07:54:22Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-06-25T06:31:02Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** NO-GO
- **Rationale:** Recommendation: DEFER

Rationale:

R5 is the arc-002 observable-governance design node (Lock 2, after R4 shipped). Q2 already set direction (local-first capture+aggregate; daily AGGREGATED push riding R4's durable queue; tiered retention). This inception fleshes the design at decision-depth and reconciles an internal plan tension (R5 one-liner says 'NOT-over-TermLink' while Q2 resolved to a daily aggregated push OVER TermLink). DEFER pending the design write-up + the human's GO/NO-GO on transport and on whether to open the build now vs after R7 hygiene de-noises the measurement surface.

Evidence:

### 2026-06-25T06:31:02Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: NO-GO

### 2026-06-25 — decision-correction [human-directed]
- **Change:** reopened work-completed → started-work; corrected decision NO-GO → DEFER
- **Reason:** The NO-GO above was an accidental Watchtower misclick. The agent
  recommendation was DEFER, and the recorded NO-GO rationale itself read as a
  DEFER rationale (it ended "DEFER pending the design write-up…"). Human directed
  the correction.
- **Follow-up:** `revisit_at: 2026-07-25` + `revisit_evidence_needed` set per
  T-1451 so the G-053 daily scan resurfaces it. Awaiting formal re-record:
  `fw inception decide T-2250 defer --rationale "…"` (human Tier-0 action).

### 2026-07-02T15:40:51Z — status-update [task-update-agent]
- **Change:** status: started-work → captured
- **Change:** horizon: later → later
- **Reason:** T-1865 sweep: DEFER limbo recovery
