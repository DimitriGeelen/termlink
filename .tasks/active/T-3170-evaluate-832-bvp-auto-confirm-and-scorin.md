---
id: T-3170
name: "Evaluate 832 BVP auto-confirm and scoring telemetry design from pickup offset
  168"
description: >
  Inception: Evaluate 832 BVP auto-confirm and scoring telemetry design from pickup
  offset 168

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-09-26T17:52:31Z
last_update: 2026-09-26T17:52:50Z
date_finished:
revisit_at: 2026-10-03
revisit_evidence_needed: "IW-3 settled: acd_gate's actual verb list read in OUR vendored lib/bvp.sh, plus a count of tasks genuinely blocked on human confirmation (IW-5). Backstop only — the primary forcing function is the framework-pickup canary, deliberately left firing on offset 168 until this task reaches a decision."
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 3            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.5                    # float 0..1. Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
bvp_scores_proposed:
  - ts: '2026-09-26T17:52:50Z'
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
---

# T-3170: Evaluate 832 BVP auto-confirm and scoring telemetry design from pickup offset 168

## Problem Statement

<!-- What problem are we exploring? For whom? Why now? -->

The framework can score a task for priority (BVP). An agent may PROPOSE a score; only a human may
CONFIRM it, via `fw bvp confirm --i-am-human`. That is a real sovereignty gate — the scoring RULES
are policy, and policy is the operator's.

But it also means every scored task needs a personal approval of a number the operator usually has
no strong opinion about. The predictable result: scores do not get confirmed, and the priority
ranking that depends on them stays empty. The one feature meant to answer "what should I work on"
is starved by the gate protecting it.

832's operator ruled on this for their project ("scoring just happens ... key thing is we want
telemetry about it"), they built it, and they sent the DESIGN here rather than keeping it local,
noting it is a change to our governance model and that their ruling binds them only.

**Why now:** the filing has been unanswered on `framework:pickup` since it arrived, and the
framework-pickup canary has been firing on it the whole time. The alarm is the only reason it
reached the operator.

**This task demonstrates its own IW-4 concern.** The estimator scored T-3170 itself
`D1=2 D2=2 D3=2 D4=2 F-RECALL=2 F-ORCH=2`, every driver "no-signal", normalising to exactly 0.40 —
on a task whose body carries a full problem statement, five questions and a four-step plan. Under
832's ledger design, a confirmation of that would be recorded as `proposer_exact`: the estimator
agreeing with its own default, logged as evidence the estimator is accurate. That is the
measurement-that-cannot-fail IW-4 exists to test for, and it is reproducible on this file.

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

- **IW-1: Is this termlink's call or a framework-wide one?**
  confidence: 1
  disposition: answered|deferred|dissolved
  rationale: <working assumption: framework-wide. `lib/bvp.sh` is vendored, so a local patch is deleted by the next re-vendor — the exact failure 832 designed around by using a config switch. Operator owns both, so the scope is theirs to set; recorded as an assumption, not a settled fact.>

- **IW-2: Telemetry slice first, or the switch first?**
  confidence: 3
  disposition: answered
  rationale: NEITHER - my telemetry-first assumption is REFUTED by IW-5's measurement. The ledger records confirmations; there have been 0 confirmations ever and there will be 0 more while the gate stands, so a telemetry-first slice produces an empty file indefinitely. The halves are coupled. Correct order is both-at-once behind the off-by-default switch, telemetry landing first WITHIN that slice so no confirmation is written unrecorded. See report F5.

- **IW-3: Are 832's claims about OUR tree actually true?**
  confidence: 3
  disposition: answered
  rationale: VERIFIED by reading our own lib/bvp.sh - acd_gate guards exactly five verbs (weight :650, confirm :858, driver --add :939, driver --remove :1233, auto-promote --enable :1347) and fw arc create has no agent gate. Bonus finding F2: the gate's own refusal text cites 'weight/driver changes carry policy-edit authority' while refusing `confirm`, which is neither - our source already concedes their distinction.

- **IW-4: Does the telemetry design survive OUR estimator's behaviour?**
  confidence: 3
  disposition: answered
  rationale: MEASURED - 74 of 449 proposals (16%) are all-2s/fully no-signal, the vacuous rows. 84% carry real signal, so the contaminant is bounded, not fatal. AMENDMENT REQUIRED: the ledger must record the no-signal count per row, else 16% of rows silently inflate any accuracy figure computed from it.

- **IW-5: What would make us NOT do this?**
  confidence: 3
  disposition: answered
  rationale: The NO-GO condition is NOT met - the bottleneck is total. Across 2877 tasks: 449 carry a proposed score, 0 have EVER been confirmed. A 100% block rate over the whole corpus, so every BVP-derived surface has never held a single data point. This is the strongest evidence in the task and it points to GO.

## Exploration Plan

<!-- How will we validate assumptions? Spikes, prototypes, research? Time-box each. -->

1. **Read our own acd_gate** (30 min, pure read). Enumerate the verbs it actually guards in our
   vendored `lib/bvp.sh`; confirm or refute 832's five-verb claim and the `fw arc create` claim
   at lib/arc.sh:399. Settles IW-3. Nothing else starts until this is done.
2. **Measure the bottleneck** (30 min, pure read). Count tasks carrying `bvp_scores_proposed:`
   with no confirmed `bvp_scores:`. That number IS the problem statement; if it is small, IW-5
   returns NO-GO and the rest is moot.
3. **Measure estimator discrimination** (45 min, pure read). Over tasks that DO carry proposals,
   what fraction are all-2s / normalise to exactly 0.40? That is the share of ledger rows that
   would be vacuously `proposer_exact`. Settles IW-4.
4. Only then: scope decision (IW-1) and a recommendation. No code is written under this task.

## Technical Constraints

<!-- What platform, browser, network, or hardware constraints apply?
     For web apps: HTTPS requirements, browser API restrictions, CORS, device support.
     For hardware APIs (mic, camera, GPS, Bluetooth): access requirements, permissions model.
     For infrastructure: network topology, firewall rules, latency bounds.
     Fill this BEFORE building. Discovering constraints after implementation wastes sessions. -->

## Scope Fence

<!-- What's IN scope for this exploration? What's explicitly OUT? -->

**IN:** reading our own vendored `lib/bvp.sh` and `lib/arc.sh` to verify 832's claims; counting the
real confirm bottleneck; measuring how often our estimator emits an all-no-signal proposal; a scope
recommendation and a go/no-go.

**OUT — and these are firm:**
- **No code.** This inception writes no implementation, no switch, no telemetry writer.
- **No local patch to `lib/bvp.sh`.** It is vendored (G-062); a local edit is deleted by the next
  re-vendor, which is the exact failure 832 designed around.
- **No acking of offset 168.** Deliberate — see the research artifact. The canary is the forcing
  function and it stays firing until this reaches a decision.
- **No touching the other four gated verbs.** `weight --set`, `driver --add/--remove`,
  `auto-promote --enable` edit the value model itself; the ruling did not cover them and neither
  does this task.

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
- 832's three claims about acd_gate / the five verbs / arc create are VERIFIED against our own vendored tree (IW-3)
- The confirm bottleneck is measurably real here — a countable number of tasks actually blocked on human confirmation (IW-5)
- The telemetry can be shown to discriminate, i.e. our estimator's no-signal default does not make `proposer_exact` vacuously true (IW-4)
- A scope decision exists on whether this lands framework-side rather than as a vendored local patch the next re-vendor deletes (IW-1)

**NO-GO if:**
- The bottleneck is not real here — few or no tasks are genuinely blocked on confirmation, so the change removes a gate for no gain (IW-5)
- The telemetry cannot distinguish a real estimate from the estimator's 2-everywhere default, making the ledger a measurement that cannot fail (IW-4)
- Our acd_gate differs enough from theirs that the design does not transfer, and re-deriving it costs more than the bottleneck does
- Note: "it is only implementable upstream" is NOT a NO-GO — it is a scope answer to IW-1, and the filing goes upstream either way

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

832 contributed a design, not a request: auto-confirm behind an off-by-default switch for the single confirm verb, plus a telemetry ledger recording proposed-vs-confirmed per driver. Their claims about which verbs acd_gate guards, and about fw arc create having no agent gate, are unverified here. DEFER until those are checked against our own vendored tree and the telemetry slice is scoped.

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

<!-- Filled at completion via: fw inception decide T-XXX go|no-go --rationale "..." -->

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-09-26T17:52:50Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
