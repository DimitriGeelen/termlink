---
id: T-2995
name: "Inception: RPC-surface wiring-check sweep (orchestrator.route, dialog.presence,
  session.*, event.*)"
description: >
  S-21/C-27,C-28,C-31,C-32,C-33: sweep RPC surfaces for wired-vs-orphaned status;
  includes the event.broadcast reference sweep that may convert C-33 into a clean
  DELETE candidate. C-27 final ruling is human. Evidence: docs/reports/VALUE-REVIEW-repo-2026-09-19-consolidated.md
  C-27..C-33.

status: started-work
workflow_type: inception
owner: agent
horizon: now
tags: [value-review, arc:arc-009]
components: []
related_tasks: []
created: 2026-09-19T22:18:28Z
last_update: 2026-09-20T21:55:00Z
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
  - ts: '2026-09-20T08:45:11Z'
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
    rubric_sha: e4a00f38e801
cost_estimate_proposed:
  - ts: '2026-09-20T08:45:20Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 4
      effort: 6
    rationale: blast_radius=3 (target_blast_radius:inception-T-2189); tier=4 
      (workflow:inception); effort=6 (lines=115,acs=4)
    rubric_sha: e4a00f38e801
---

# T-2995: Inception: RPC-surface wiring-check sweep (orchestrator.route, dialog.presence, session.*, event.*)

## Problem Statement

<!-- What problem are we exploring? For whom? Why now? -->

Five RPC surfaces are recorded as hub-implemented (or CLI/MCP-surfaced) with **zero
observed calls**: `orchestrator.route` (C-27), `dialog.presence` (C-28), the `session.*`
lifecycle methods (C-31), the `event.*` family (C-32), and `event.broadcast` (C-33).
Each is a candidate for either WIRE (it was wanted and never connected) or DELETE (it is
resident code nobody can reach) — and the review held all five at INVESTIGATE because
nobody had run the per-method wiring check.

For whom: anyone reading the RPC surface as a statement of what the system does. Why now:
C-33 is a declared retirement (`LEGACY_METHODS`, "targeted for retirement") that failed
DELETE checks 4-5 only because the reference sweep was never run, so it sits in a state
where the code says "going away" and the process says "not yet".

**The prior that gates everything else:** C-30 found `kv.*` usage structurally invisible —
session-daemon calls never pass the hub audit sink — and C-31 records that the same blind
spot "plausibly applies (not individually verified)". If it does, "zero calls" in these
rows measures the instrument and not the traffic.

Full measurements: `docs/reports/T-2995-rpc-surface-wiring-sweep.md`.

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

- **IW-1: For each of the five surfaces (orchestrator.route, dialog.presence, session.*, event.* family, event.broadcast), is it hub-implemented, and does it carry a client surface (CLI verb and/or MCP tool)?**
  confidence: 3
  disposition: answered
  rationale: Matrix measured in source — orchestrator.route and dialog.presence: hub arm, zero CLI/MCP; session.* lifecycle: no hub arm at all (served by termlink-session/src/handler.rs); event.* splits three ways (live / daemon-served / orphan constants). Hub router carries 64 arms, only 5 are session.*/event.* — artifact F2

- **IW-2: Does the C-30 audit blind spot apply to session.* and event.* — i.e. is "zero hub-observed calls" evidence of disuse, or evidence of nothing?**
  confidence: 3
  disposition: answered
  rationale: Yes, and by construction: server.rs:1610 is the ONLY general dispatch audit site, and termlink-session/src/handler.rs — which serves session.register/deregister/heartbeat/update — has ZERO audit references. Second blind spot found: rpc_audit.rs:47 SKIP_METHODS excludes event.poll/event.collect by design (T-1307). Every zero-call reading in C-31/C-32 is inadmissible — artifact F1

- **IW-3: Does the C-33 event.broadcast reference sweep (DELETE checks 4-5: references, external consumers) come back clean?**
  confidence: 3
  disposition: answered
  rationale: Clean of callers, but the question was already moot: event.broadcast was CUT 2026-05-31 (router.rs:1017, T-1166/T-1415) and the hub returns -32601. Residue is the constant + a LEGACY_METHODS warn entry + tests/no_legacy_callers.rs which enforces the retirement by naming it — artifact F3

- **IW-4: Does anything depend on orchestrator.route remaining present — specifically the federation tripwire C-27 warns not to break?**
  confidence: 3
  disposition: answered
  rationale: No — and the tripwire says so itself: no_federation_tripwire.rs:43 names orchestrator.route as the residual path that 'is NOT covered and cannot be by a static check'. What DOES exist is a 350-line handler, a dedicated route_cache module (Layer 3 of 3), and a live E2E driver at tests/e2e/level8-orchestration-harness.sh:121 — artifact F4

## Exploration Plan

<!-- How will we validate assumptions? Spikes, prototypes, research? Time-box each. -->

All spikes are read-only reads of the working tree. No RPC method is added, wired,
removed, or renamed by this task.

1. **Blind-spot first** — establish whether the hub audit sink observes `session.*` and
   `event.*` at all, by reading the audit call sites rather than the call counts. This
   runs FIRST because it decides whether the zero-call evidence in steps 2-3 is
   admissible → IW-2.
2. **Wiring matrix** — for each of the five surfaces: hub dispatch arm present? CLI verb
   present? MCP tool present? → IW-1.
3. **Reference sweep** — `event.broadcast` across source, config, hooks, CI, docs and
   prompts; DELETE checks 4-5 → IW-3.
4. **Tripwire check** — what reads `orchestrator.route`, and does any guard depend on its
   presence → IW-4.

## Technical Constraints

<!-- What platform, browser, network, or hardware constraints apply?
     For web apps: HTTPS requirements, browser API restrictions, CORS, device support.
     For hardware APIs (mic, camera, GPS, Bluetooth): access requirements, permissions model.
     For infrastructure: network topology, firewall rules, latency bounds.
     Fill this BEFORE building. Discovering constraints after implementation wastes sessions. -->

## Scope Fence

<!-- What's IN scope for this exploration? What's explicitly OUT? -->

**IN:** measuring, per surface, whether it is hub-implemented and client-reachable;
establishing whether the C-30 audit blind spot extends to these families; running the
C-33 reference sweep; identifying what depends on `orchestrator.route`.

**OUT — deliberately:**
- Deleting, wiring, or renaming any RPC method. This is an inception; the sweep is read-only.
- The **C-27 ruling**. Whether `orchestrator.route` may exist at all turns on charter
  non-goal #4, which is a sovereignty question. The measurement is agent work; the ruling
  is not, and is surfaced rather than resolved.
- Building the C-45/C-30 telemetry instrument. That is T-2996's scope, and this task
  depends on its ABSENCE being characterised, not on it being built.
- Deciding go/no-go. The `### Human [REVIEW]` AC owns that.

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

**Recommendation:** GO — with a result materially different from the one this section
carried before the exploration ran.

**Rationale:** The sweep's main finding is that three of the five rows were asking the
wrong question.

- **C-31 and most of C-32 rest on inadmissible evidence.** The session daemon
  (`termlink-session/src/handler.rs`), which serves the entire `session.*` lifecycle, has
  **no audit sink at all**, and `rpc_audit.rs:47` excludes `event.poll`/`event.collect` by
  design. "Zero hub-observed calls" there is structurally guaranteed regardless of traffic.
  C-31 asked for this verification; it comes back positive, so its own disposition is
  *decide nothing on usage until T-2996 lands*.
- **C-33's premise is already spent.** `event.broadcast` was cut **2026-05-31**
  (`router.rs:1017`, T-1166/T-1415); the hub returns `-32601`. The reference sweep is clean,
  but what remains is residue held in place by `no_legacy_callers.rs`, which enforces the
  retirement by naming the constant.
- **C-27's reading is refuted.** `orchestrator.route` is not shelved scaffolding: 350-line
  handler, a dedicated `route_cache` module describing it as Layer 3 of three, and a live
  E2E harness driving it. It lacks a *client surface*, which is what the zero-call reading
  actually measures. The federation tripwire does not depend on it —
  `no_federation_tripwire.rs:43` names it as the path it cannot cover.
- **C-28 is cheap and clean.** `dialog.presence` has handler, tests, capability
  advertisement, and a live producer maintaining state *for a query nobody can issue*
  (`channel.rs:941`). Only the client surface is missing.
- **The real clean DELETEs were never in the review.** `event.state_change` and
  `event.error` have zero references outside their own definition in `control.rs` — no hub
  arm, no CLI, no MCP, no test. They are the only surfaces here removable on structure
  alone, so F1's inadmissibility does not touch them.

**Dependency-ordered scope:** (1) remove the two orphan constants — independent of
everything. (2) wire `dialog.presence`. (3) retire the `event.broadcast` residue as one unit
with its guard-test expectations. (4) `orchestrator.route` — **human**: the measurement
supports WIRE and does not support DELETE, but whether it may exist turns on non-goal #4.
(5) `session.*` and daemon-served `event.*` — **blocked on T-2996**; no usage disposition
before the telemetry exists.

**Note on the prior text:** this section arrived pre-filled with "GO" before any spike ran,
predicting the sweep would "resolve five INVESTIGATE rows and may produce the review's only
clean DELETE". It resolves two; two are blocked on T-2996 and one on the human. And C-33 —
the predicted clean DELETE — was retired four months before the prediction was written,
while the actual clean DELETEs are two constants the review never identified. The GO
survives; none of its stated reasons did. **This is the third consecutive arc-009 inception
(after T-2989 and T-3001) whose Recommendation arrived pre-filled with a conclusion the
exploration then contradicted.**

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

### 2026-09-19T22:35:32Z — status-update [task-update-agent]
- **Change:** tags: +arc:arc-009

### 2026-09-20T21:53:49Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)
