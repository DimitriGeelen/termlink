---
id: T-3524
name: "wc/0.3: which remaining vector capabilities are hub obligations"
description: >
  Inception: wc/0.3: which remaining vector capabilities are hub obligations

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-10-21T14:44:17Z
last_update: 2026-10-22T09:52:22Z
date_finished:
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 3            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.6                    # float 0..1. Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
                                  # T-3172: deliberately EMPTY — set a considered value. 0.5 (and 0.3/0.4) score 2, the same as "not assessed".
bvp_scores_proposed:
  - ts: '2026-10-22T08:32:39Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 3
      D2: 3
      D3: 3
      D4: 3
      F-RECALL: 3
      F-ORCH: 3
    rationale: D1=3 (no-signal); D2=3 (no-signal); D3=3 (no-signal); D4=3 
      (no-signal); F-RECALL=3 (no-signal); F-ORCH=3 (no-signal)
    rubric_sha: e4a00f38e801
---

# T-3524: wc/0.3: which remaining vector capabilities are hub obligations

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

- **IW-1: Is a bounded `wait` on a dispatch record (CC-08) a hub surface, or a client poll of `dispatch.get`?**
  confidence: 3
  disposition: answered
  rationale: hub surface `dispatch.wait` (operator A′1, 2026-10-22; codex and antigravity agree; no lost wakeup, defined across restart)
- **IW-2: For a hub backend, is a partition (CC-09) the existing absent-registration rule (two sweeps + G, `probe_undecided`), or does the hub need its own worker-host contact model?**
  confidence: 3
  disposition: answered
  rationale: neither — the absent-registration mapping is rejected (both reviewers: visibility loss ≠ execution loss); partition/heal is adapter fault injection that replays a late done against the fencing (operator A′1)
- **IW-3: Are the parent-session-loss queue entry and `adopt` (CC-10) hub obligations (spec §4.6), or dispatcher tooling outside the hub?**
  confidence: 3
  disposition: answered
  rationale: hub obligations (§4.6): orphan routing of questions and bounces, plus an authorised, atomic, generation-advancing `adopt` (operator A′1; both reviewers)
- **IW-4: Should `hold_lock` (CC-13) be exercised through the adapter against the real SQLite lock, or is T-3531's unit coverage the accepted evidence?**
  confidence: 3
  disposition: answered
  rationale: through the adapter against the real SQLite lock, short test timeout (operator A′1; both reviewers)
- **IW-5: Are `consume` refusals (CC-19, CC-22) AEF's evaluation layer (n/a for a backend), or should the hub derive a `finished_work` fact?**
  confidence: 3
  disposition: answered
  rationale: AEF's evaluation layer; dispatch.get documents a normative finished_work rule (operator A′1 over A′2's hub dispatch.consume gate; codex for, antigravity against)

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

**Recommendation:** GO (option A′1, operator ruling 2026-10-22, "Suggested Proceed"; stamp pending in Watchtower)

**Rationale:**

Build in the hub: `dispatch.wait` (bounded; no lost wakeup; defined across hub restart; never waits inside a write
transaction); orphan routing of questions **and** bounces when the dispatcher session is not live; `adopt`, authorised
and atomic, advancing the fencing generation and carrying pending questions and bounces with deadlines unchanged.
Build in the adapter: a `hold_lock` hook on the real SQLite lock (short test timeout); `partition`/`heal` as fault
injection that replays a late `done` to prove fencing rejects a stale completion. The grading refusal (`consume`,
CC-19/CC-22) stays with AEF's evaluation layer; `dispatch.get` documents a normative `finished_work` rule. External
review: docs/reports/T-3524-consult/ (codex, antigravity; both "modified A"; they differ only on consume).

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

### 2026-10-22T08:32:38Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
