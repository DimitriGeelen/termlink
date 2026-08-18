---
id: T-2477
name: "Shipped-equals-live: arc-closure capability gate + make-it-live primitive (T-2468
  P3, G-069)"
description: >
  Inception: Shipped-equals-live: arc-closure capability gate + make-it-live primitive
  (T-2468 P3, G-069)

status: work-completed
workflow_type: inception
owner: human
horizon:
tags: []
components: []
related_tasks: []
created: 2026-08-01T19:50:32Z
last_update: '2026-08-18T18:59:11Z'
date_finished: 2026-08-01T21:08:26Z
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 3            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.5                    # float 0..1. Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:47Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 2
      D2: 2
      D3: 2
      D4: 2
      F-RECALL: 2
      F-ORCH: 2
    rationale: D1=2 (no-signal); D2=2 (no-signal); D3=2 (no-signal); D4=2 
      (no-signal); F-RECALL=2 (no-signal); F-ORCH=2 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:11Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 4
      effort: 8
    rationale: blast_radius=3 (no-signal); tier=4 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2477: Shipped-equals-live: arc-closure capability gate + make-it-live primitive (T-2468 P3, G-069)

## Problem Statement

TermLink repeatedly records a capability as `closed=shipped` while it is **dark in
the field** for weeks (G-069). Concrete evidence: ring20-management's hub served a
13-day-old *deleted-inode* binary lacking the arc-004 push rails while the arc was
"closed"; the .107 hub itself was 26 commits stale during that same "closure". The
word "shipped" silently conflates **code-merged** with **capability-live**.

Two structural holes cause this, for the fleet operator:

1. **No arc-closure capability gate.** An arc/task can reach `work-completed` / arc
   `closed=shipped` with zero proof that the primary hub actually *serves* the new
   capability. The verification gate (P-011) runs local build/test commands, not a
   live-capability probe against the deployed hub.
2. **No make-it-live primitive.** Fleet adoption is fully manual, per-host, and
   foothold-gated (`fleet-deploy-binary.sh` is single-host; off-.107 hosts need an
   operator session). There is no one-command "roll this capability live across the
   reachable fleet and confirm it" verb. The 11 cron canaries are the *symptom* of
   this blindness — each one exists because nothing made adoption observable-by-default.

Detection is now *partially* covered (T-2359 fleet-binary-freshness canary, T-2387
waker-liveness, T-2415 capability-freshness) — but detection-after-the-fact is not
the same as a *gate* that refuses to call something "shipped" until it is live, nor
a *primitive* that makes going-live cheap. Why now: T-2468 IW-4 ranked this the
highest-leverage adoption gap, and the fleet is at a rare green steady-state (all
reachable hubs doorbell-capable) — the right moment to add the gate before the next
arc closes against a stale hub.

## Assumptions

<!-- Key assumptions to test. Register with: fw assumption add "Statement" --task T-XXX -->

## Open Questions

- **IW-1: Should "shipped" carry a mandatory capability-live gate, and at what layer
  (arc-close vs task-complete vs both)?**
  confidence: 1
  disposition: deferred
  rationale: Bounded, agent-scoped candidate — a one-line live-probe added to the
  arc-closing task's `## Verification` (e.g. `termlink fleet doctor` / a cv-keys
  capability probe against the primary hub). Needs the human to decide whether it is
  mandatory (blocks close) or advisory (warns). Design, then human GO.

- **IW-2: Is a "make-it-live" primitive a genuine new subsystem or a thin composition
  of existing verbs (fleet-deploy-binary + restart-through-systemd + capability probe)?**
  confidence: 1
  disposition: deferred
  rationale: If thin composition → agent-scoped script. If it must reach foothold-less
  hosts (.121/.141) → new fleet-orchestration surface (new subsystem → human GO per
  Autonomous Mode Boundaries). Scope must be split before building.

- **IW-3: Are the 11 canaries reducible once a live-gate exists, or are they
  independent value?**
  confidence: 1
  disposition: deferred
  rationale: If the gate makes going-live observable-by-default, several detection
  canaries become redundant — but retiring a canary is its own reversible decision.
  Out of scope for the first build; note for a later grooming pass.

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

<!-- How will we validate assumptions? Spikes, prototypes, research? Time-box each. -->

## Technical Constraints

<!-- What platform, browser, network, or hardware constraints apply?
     For web apps: HTTPS requirements, browser API restrictions, CORS, device support.
     For hardware APIs (mic, camera, GPS, Bluetooth): access requirements, permissions model.
     For infrastructure: network topology, firewall rules, latency bounds.
     Fill this BEFORE building. Discovering constraints after implementation wastes sessions. -->

## Scope Fence

IN: designing the arc-closure capability-live gate (where it lives, mandatory vs
advisory, what it probes); scoping whether a make-it-live primitive is thin
composition or a new subsystem; a GO/NO-GO recommendation per sub-part.
OUT: building the make-it-live primitive if it proves to be a new fleet-orchestration
subsystem (that needs its own human GO); upgrading foothold-blocked hosts (.121/.141);
retiring existing canaries (separate reversible decisions, IW-3).

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
- Root cause identified with bounded fix path
- Fix is scoped, testable, and reversible

**NO-GO if:**
- Problem requires fundamental redesign or unbounded scope
- Fix cost exceeds benefit given current evidence

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

T-2468 IW-4 found the 'shipped != live' class (G-069): capabilities recorded closed=shipped while dark in the field for weeks (.122 served a 13-day-old deleted-exe binary; .107 itself 26 commits stale during arc-004 'closure'). Detection is now partly covered (T-2359 fleet-binary canary, T-2387 waker-liveness) but the ROOT remains: (1) arc-closure has no capability-live gate — 'shipped' still means code-merged not capability-live; (2) no make-it-live primitive — fleet adoption is fully manual/per-host/foothold-gated. GO to inception: the arc-closure gate is a bounded template change (agent-scoped); the make-it-live primitive is a new fleet-orchestration surface (human GO). Split needed; design first.

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

T-2468 IW-4 found the 'shipped != live' class (G-069): capabilities recorded closed=shipped while dark in the field for weeks (.122 served a 13-day-old deleted-exe binary; .107 itself 26 commits stale during arc-004 'closure'). Detection is now partly covered (T-2359 fleet-binary canary, T-2387 waker-liveness) but the ROOT remains: (1) arc-closure has no capability-live gate — 'shipped' still means code-merged not capability-live; (2) no make-it-live primitive — fleet adoption is fully manual/per-host/foothold-gated. GO to inception: the arc-closure gate is a bounded template change (agent-scoped); the make-it-live primitive is a new fleet-orchestration surface (human GO). Split needed; design first.

Evidence:

**Date**: 2026-08-01T21:08:25Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-08-01T19:51:04Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-08-01T21:08:25Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Recommendation: GO

Rationale:

T-2468 IW-4 found the 'shipped != live' class (G-069): capabilities recorded closed=shipped while dark in the field for weeks (.122 served a 13-day-old deleted-exe binary; .107 itself 26 commits stale during arc-004 'closure'). Detection is now partly covered (T-2359 fleet-binary canary, T-2387 waker-liveness) but the ROOT remains: (1) arc-closure has no capability-live gate — 'shipped' still means code-merged not capability-live; (2) no make-it-live primitive — fleet adoption is fully manual/per-host/foothold-gated. GO to inception: the arc-closure gate is a bounded template change (agent-scoped); the make-it-live primitive is a new fleet-orchestration surface (human GO). Split needed; design first.

Evidence:

## Reviewer Verdict (v1.5)

- **Scan ID:** R-025d76cb
- **Timestamp:** 2026-08-01T21:08:27Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Verification-level findings:**

  1. **disposition-incomplete** (partial, heuristic) @ ## Open Questions: IW-1
     - evidence: `IW-1 disposition='answered' but rationale has no evidence citation (T-NNNN, file:line, docs/reports/, G-/L-/D-id, dialogue-log, or commit hash)`

### 2026-08-01T21:08:26Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
