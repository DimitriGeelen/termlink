---
id: T-2879
name: "Rail enrolment gap: 1 of 14 local Claude sessions is on the TermLink presence
  rail"
description: >
  T-2875 measured two working reach rails and disproved every assumption it carried
  in, concluding that nothing needs BUILDING for cross-session reach. The gap it surfaced
  instead is ENROLMENT: 14 Claude Code sessions are live on .107 and exactly 1 appears
  on TermLink agent-presence. Question: how many of the 13 SHOULD have enrolled, and
  if the answer is more than a couple, is the fix a launcher default (scripts/tl-claude.sh
  --reachable, T-2388), a prompt-time nudge, or an accepted cost? PL-237 constrains
  the answer space - reach must be arranged at LAUNCH, so no retrofit path exists
  and any fix is a launch-path fix. Start by reading what fleet-adoption-snapshot
  already records rather than measuring afresh.

status: started-work
workflow_type: inception
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-09-02T06:19:43Z
last_update: 2026-09-25T21:46:12Z
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
  - ts: '2026-09-08T21:30:31Z'
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
  - ts: '2026-09-08T21:30:40Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 4
      effort: 6
    rationale: blast_radius=3 (target_blast_radius:inception-T-2189); tier=4 
      (workflow:inception); effort=6 (lines=112,acs=4)
    rubric_sha: e4a00f38e801
---

# T-2879: Rail enrolment gap: 1 of 14 local Claude sessions is on the TermLink presence rail

## Problem Statement

<!-- What problem are we exploring? For whom? Why now? -->

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

- **IW-1: Is the premise still true — is roughly 1 of ~14 local Claude sessions on the presence rail today?**
  The figure comes from arc-011's S10 note and is ~2 weeks old; the fleet has changed since. An inception that reasons from a stale measurement recommends a fix for a condition that may no longer exist.
  confidence: 0
  disposition: answered
  rationale: Measured 2026-09-25: 1 of 9 REACHABLE sessions on the rail (84 records, 76 at-rest), and that 1 carries no pty_session — docs/reports/T-2879-rail-enrolment-gap.md F1.

- **IW-2: What is the MECHANISM that keeps a session off the rail — opt-in friction, a launcher gap, or something structural about how sessions are started?**
  PL-237 records that a running headless claude cannot be retrofitted onto the rail (it must be armed at launch). If enrolment is only possible at spawn time, then any remediation is about how sessions are LAUNCHED, not about a verb an agent can call — which changes what a GO would even authorise.
  confidence: 1
  disposition: answered
  rationale: Launch-time only. PL-237: a running headless claude cannot be retrofitted; enrolment is a property of how the session was spawned — F4.

- **IW-3: Is the gap a defect at all, or the designed state?**
  CLAUDE.md says opting in is explicitly optional ("Skip on throw-away sessions (<2 min) or hosts that should not appear on the fleet"). If most of the 14 are finished dispatch workers sitting at a bash prompt, low enrolment is correct behaviour and the real finding is that the DENOMINATOR is wrong — we would be measuring against a population that was never meant to enrol.
  confidence: 1
  disposition: answered
  rationale: Both. 76/84 are at-rest and correctly absent (CLAUDE.md says opting in is optional); the defect is that ZERO live agents are wakeable — F5.

- **IW-4: Does closing the gap require an operator/architectural decision this run must surface rather than make?**
  Auto-enrolling every session changes fleet-wide default behaviour and has a privacy/footprint dimension (sessions appearing on a shared rail by default). That is a sovereignty call, not an implementation detail.
  confidence: 2
  disposition: deferred
  rationale: Defaulting --reachable in tl-claude.sh changes fleet-wide behaviour and has a footprint/privacy dimension — operator policy, surfaced not decided.

## Exploration Plan

<!-- How will we validate assumptions? Spikes, prototypes, research? Time-box each. -->

## Technical Constraints

<!-- What platform, browser, network, or hardware constraints apply?
     For web apps: HTTPS requirements, browser API restrictions, CORS, device support.
     For hardware APIs (mic, camera, GPS, Bluetooth): access requirements, permissions model.
     For infrastructure: network topology, firewall rules, latency bounds.
     Fill this BEFORE building. Discovering constraints after implementation wastes sessions. -->

## Scope Fence

<!-- What's IN scope for this exploration? What's explicitly OUT? -->

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

**Recommendation (2026-09-25, after measurement):** NO-GO on any build — **dissolve into T-2389**.

**Rationale:** The DEFER below was correct as written and asked for exactly one thing: partition the sessions into should-have-enrolled versus correctly-abstained before proposing anything. That partition has now been done (`docs/reports/T-2879-rail-enrolment-gap.md`), and it answers the question rather than refining it.

Measured on 2026-09-25: **84** local Claude session records, of which **9** are reachable by the T-2876 predicate and **76** are `blocked` (at rest). Those 76 are correctly absent — CLAUDE.md says opting in is optional — so the original 1-of-14 framing flattered the denominator. The defensible statement is narrower and worse: **of the agents live enough to be worth reaching, ZERO are wakeable.** The one rail entry (`penelope`) carries no `pty_session`.

The hypothesis worth killing was "the framework is blind to this". **It is not.** The T-2387 waker-liveness canary fires correctly, naming both classes (`LIVE-no-waker` and `rail-dark`), and `/canaries` agrees — `waker-liveness-canary FIRING`, 36,053-byte log. Detection and presentation are consistent.

The actual finding is the inverse of this repo's usual one: **42 `rail-dark` firings in the log — roughly six weeks of a daily cron reporting the condition correctly, with no action taken.** Loud and unheeded, which is alarm fatigue (T-2818/T-2833) seen from the receiving end.

There is nothing to build. PL-237 forecloses a retrofit (reach must be arranged at LAUNCH), and the remediation the canary itself prints — relaunch through the T-2388 launcher — is already ticketed as **T-2389** (`started-work`, arc-007, `owner: human`). Filing build work here would add a fourth artefact describing what three already describe and would not make one agent wakeable.

**Note on the criteria above:** by their letter this reads GO — the root cause is identified and the fix path is bounded. It is recorded as NO-GO/dissolve because the criteria assume the outcome is CODE, and here the bounded fix is an operator act that belongs to an existing task. Meeting "bounded fix path" is not authorisation to build something.

**Sovereign question, surfaced and deliberately not decided:** should `--reachable` become the DEFAULT in `scripts/tl-claude.sh`, so enrolment stops depending on remembering a flag? That changes fleet-wide default behaviour and puts sessions on a shared rail by default — a footprint and privacy call that is operator policy, not an implementation detail.

---

_Superseded rationale (at capture, before measurement):_ DEFER because the headline number may not be a defect. CLAUDE.md's own session rules say to skip /be-reachable on throw-away sessions and on hosts that should not appear on the fleet, so an unknown share of the 13 non-enrolled sessions are behaving exactly as documented. Filing at DEFER rather than GO keeps this from becoming a fix in search of a problem: the first move is to partition the 14 into should-have-enrolled versus correctly-abstained, using what fleet-adoption-snapshot already records, and only then ask whether a launcher default is warranted. What makes it worth filing at all rather than dropping is the failure DIRECTION - a rail nobody joined is indistinguishable from a rail that does not exist, and T-2875 caught a .122 peer making precisely that inference from an empty ListAgents. So even a fully-correct low enrolment number has a real cost that is worth naming.

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

### 2026-09-25T21:46:12Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)
