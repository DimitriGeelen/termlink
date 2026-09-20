---
id: T-3001
name: "Inception: presence-heartbeat truthfulness design (DELIVERED/BLOCKED/ENQUEUED
  vocabulary)"
description: >
  S-27/C-10: presence heartbeats overstate reachability; design a truthfulness vocabulary
  aligned with the T-2876 prover verdict set. Evidence: docs/reports/VALUE-REVIEW-repo-2026-09-19-consolidated.md
  C-10.

status: started-work
workflow_type: inception
owner: agent
horizon: now
tags: [value-review, arc:arc-009]
components: []
related_tasks: []
created: 2026-09-19T22:23:58Z
last_update: 2026-09-20T21:22:01Z
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

# T-3001: Inception: presence-heartbeat truthfulness design (DELIVERED/BLOCKED/ENQUEUED vocabulary)

## Problem Statement

A presence heartbeat on `agent-presence` is read across this repo as "this agent is
reachable". It is not that claim. It is the weaker claim that a process was alive and
emitting N seconds ago. Between those two sits every failure T-2873/74/75 found in one
week: a hub reporting `injected` while the PTY got nothing, a config that looked
authoritative and was never read, and a send that succeeded while the target sat blocked.

T-2876 already built the vocabulary that tells those apart on the MESSAGE rail --
DELIVERED / BLOCKED / ENQUEUED / UNDELIVERED, asserted on the receiver and never on the
sender. Presence is the remaining surface that still asserts something it cannot observe,
and its consumers (`/peers`, `agent find-idle`, doorbell discovery) treat LIVE as
dispatchable.

For whom: every orchestrator that picks a worker off presence. Why now: the truthfulness
vocabulary exists and is proven, so the question is whether it transfers to this surface,
not whether such a vocabulary is possible.

Findings artifact: `docs/reports/T-3001-presence-heartbeat-truthfulness.md`.

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

- **IW-1: What does a presence heartbeat actually assert today — which fields does the producer emit, and what does each consumer infer from them?**
  confidence: 3
  disposition: answered
  rationale: Ten metadata fields, ALL bound before the loop (listener-heartbeat.sh:150-157 vs post_one); consumers apply a pure recency test (agent-listeners.sh:13-16). LIVE = 'a bash loop iterated recently' — artifact F1/F2

- **IW-2: Can an agent be LIVE on `agent-presence` while genuinely unable to act on a message — does the T-2876 BLOCKED class have a presence-side counterpart that is currently invisible?**
  confidence: 3
  disposition: answered
  rationale: Yes, and by construction: be-reachable.sh:253-264 spawns the beater `nohup setsid ... & disown` 'so it survives this shell exit', so presence measures a different process than the one that would act; state-file pid is the beater's ($!), pid_alive() validates the beater — artifact F3

- **IW-3: Does the T-2876 verdict set (DELIVERED/BLOCKED/ENQUEUED/UNDELIVERED) transfer to presence, or is presence a different axis needing its own vocabulary?**
  confidence: 2
  disposition: answered
  rationale: Partial: BLOCKED and ENQUEUED map exactly onto the two states LIVE collapses; DELIVERED/UNDELIVERED are message-scoped and do not transfer. The PRINCIPLE (emit only what the emitter observes) transfers fully. States measured; mapping reasoned, hence confidence 2 — artifact F4

- **IW-4: Where would truthfulness be enforced — producer-side (emit only what the process can observe) or consumer-side (stop over-reading LIVE) — and how much of each surface is vendored (G-062, upstream) vs local?**
  confidence: 3
  disposition: answered
  rationale: Fix is entirely local (scripts/ + crates/); but .agentic-framework/lib/templates/scripts/ carries copies that seed every bootstrapped project, and the template listener-heartbeat.sh has the identical defect (started_at:123 emitted:137) — template half is an upstream filing per G-062 — artifact F5

## Exploration Plan

Read-only measurement against the working tree. No producer or consumer is changed by
this inception.

1. **Producer** -- read `scripts/listener-heartbeat.sh` / `be-reachable` and enumerate
   every field a heartbeat emits, and which of them the emitting process can actually
   observe about itself -> IW-1 (producer half).
2. **Consumers** -- enumerate every reader of `agent-presence` and record the predicate
   each applies and the conclusion it draws -> IW-1 (consumer half), IW-2.
3. **Counterpart check** -- test whether a LIVE-but-cannot-act state is reachable, and
   whether anything today distinguishes it -> IW-2.
4. **Vocabulary fit** -- hold the T-2876 verdict set against the states found in 1-3 and
   record which map, which do not, and what is left over -> IW-3.
5. **Ownership** -- for each surface a fix would touch, record vendored vs local -> IW-4.

## Technical Constraints

<!-- What platform, browser, network, or hardware constraints apply?
     For web apps: HTTPS requirements, browser API restrictions, CORS, device support.
     For hardware APIs (mic, camera, GPS, Bluetooth): access requirements, permissions model.
     For infrastructure: network topology, firewall rules, latency bounds.
     Fill this BEFORE building. Discovering constraints after implementation wastes sessions. -->

## Scope Fence

**IN:** measuring what the heartbeat asserts; enumerating consumers and their predicates;
testing whether LIVE-but-blocked is reachable and invisible; assessing whether the T-2876
vocabulary transfers; recording vendored-vs-local ownership of every surface a fix touches.

**OUT -- deliberately:**
- Changing any producer or consumer. This is an inception; no behaviour is altered.
- Patching vendored code (G-062) -- anything upstream is filed, not edited here.
- Deciding go/no-go. The `### Human [REVIEW]` AC owns that, and producer-not-judge forbids
  me ratifying my own exploration.

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

**Recommendation:** GO — with a rationale materially different from the one this
section carried before the exploration ran.

**Rationale:** The premise holds but the pre-filled text mis-stated it. Presence does
not "assert delivery it cannot see" — delivery is message-scoped and presence is prior
to it. What presence asserts is *dispatchability*, and the measured defect is sharper:
`LIVE` is the union of T-2876's BLOCKED and ENQUEUED, the exact pair that vocabulary
exists to tell apart (artifact F4).

Two measurements carry it. Every heartbeat field is bound before the loop starts, so a
beat establishes only that a bash loop is iterating (F1), and consumers apply a pure
recency test to that replay (F2). And the beater is deliberately not the agent —
`be-reachable.sh:253-264` detaches it with `nohup setsid ... & disown` so it outlives
the session — so presence structurally cannot see whether the agent can act (F3).
Nothing closes this: T-2239, T-2387 and T-2405 all interrogate the delivery apparatus,
none the recipient's readiness.

**Dependency-ordered scope:** (1) rename the claim — LIVE -> BEATING at the
producer/classifier boundary, truthful under F1 with no new signal. (2) decide who
observes agent-side readiness; the beater structurally cannot, so either the agent
emits its own state or consumers stop inferring dispatchability from presence. (3) file
the vendored-template half upstream (F5) or the fix does not travel and re-imports on
the next bootstrap. (4) only then touch `find-idle`, whose anti-join treats LIVE as
dispatchable.

**Do not start (1) before (2) is decided** — renaming the field while consumers still
infer dispatchability from it relocates the untruth rather than removing it. (2) is a
design question this exploration deliberately did not settle.

Full findings: `docs/reports/T-3001-presence-heartbeat-truthfulness.md`.

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

### 2026-09-19T22:35:34Z — status-update [task-update-agent]
- **Change:** tags: +arc:arc-009

### 2026-09-20T21:20:22Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)
