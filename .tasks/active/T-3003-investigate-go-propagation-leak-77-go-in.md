---
id: T-3003
name: "Investigate GO-propagation leak (77 GO inceptions unpropagated)"
description: >
  S-29/C-35: 77 of 158 GO-recorded inceptions have empty related_tasks and no back-reference;
  investigate the leak mechanism and propose the structural fix. Evidence: docs/reports/VALUE-REVIEW-repo-2026-09-19-consolidated.md
  C-35.

status: started-work
workflow_type: inception
owner: agent
horizon: now
tags: [value-review, arc:arc-009]
components: []
related_tasks: []
created: 2026-09-19T22:25:46Z
last_update: 2026-09-20T08:49:07Z
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

# T-3003: Investigate GO-propagation leak (77 GO inceptions unpropagated)

## Problem Statement

CLAUDE.md §Inception Discipline requires that a GO decision propagate into separate build tasks
traceable back to the inception. Value-review finding C-35 measured 77 of 158 GO-recorded
inceptions with empty `related_tasks` and no back-reference from any other task — meaning for
~half of all GO decisions, whether the authorized work ever happened is undiscoverable from the
task graph. This makes the review queue and BVP prioritization blind to a whole class of
"approved but never started" work. For: the operator (review-queue truthfulness) and future
agents (selection). Why now: arc-009 executes the value review; this is its S-29a slice.

## Assumptions

- A-1: The C-35 count is reproducible on the current tree (within drift of a day's work).
- A-2: The leak is mechanical, not behavioral: no code path in `fw inception decide` (or the
  Watchtower decide route) writes a forward link at decision time, so linkage depends entirely
  on agent discipline after GO.
- A-3: A detection check (deploy-time, T-2800-tier) is buildable from existing frontmatter alone.

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

- **IW-1: Is the C-35 measurement (77/158 GO inceptions unlinked in both directions) reproducible on the current tree?**
  confidence: 3
  disposition: answered
  rationale: Reproduced 2026-09-20 as 80/167 strict (tree drifted since review); loose predicate (no body mention anywhere) collapses to 4 — see docs/reports/T-3003-go-propagation-leak-investigation.md F1/F2

- **IW-2: What is the leak mechanism — does any code path in the decide flow (update-task.sh / inception verb / Watchtower decide route) write a forward link, or is linkage purely post-GO agent discipline?**
  confidence: 3
  disposition: answered
  rationale: lib/inception.sh has zero related_tasks writes; GO prints advisory only (inception.sh:775); sole consumer is build-side (update-task.sh:917-1033) — artifact F3

- **IW-3: Is the leak ongoing or legacy — what does the date distribution of unlinked GO inceptions look like?**
  confidence: 3
  disposition: answered
  rationale: Ongoing — 2026-08 alone added 13 strict-unlinked GOs (monthly buckets in artifact F1)

- **IW-4: What is the cheapest structural fix — decide-time prompt for follow-on task IDs, a deploy-time unlinked-GO check, or both — and what does it cost?**
  confidence: 2
  disposition: answered
  rationale: Both, split by ownership: local check-go-propagation.sh + baseline ledger (T-2483 pattern, ~1 script + fixtures) here; decide-time --follow-on flag filed upstream (vendored, G-062) — artifact Recommendation

## Exploration Plan

1. Measurement script (read-only) over `.tasks/{active,completed}`: enumerate inceptions with a
   recorded GO, test forward link (`related_tasks` non-empty) and back-reference (any other task
   naming the ID). Time-box: 30 min.
2. Read the decide path in vendored `update-task.sh` / inception lib + Watchtower decide route for
   any propagation write. Time-box: 20 min.
3. Classify + date-bucket the unlinked set. Time-box: 10 min.
4. Write recommendation with fix options costed. Time-box: 20 min.

## Technical Constraints

<!-- What platform, browser, network, or hardware constraints apply?
     For web apps: HTTPS requirements, browser API restrictions, CORS, device support.
     For hardware APIs (mic, camera, GPS, Bluetooth): access requirements, permissions model.
     For infrastructure: network topology, firewall rules, latency bounds.
     Fill this BEFORE building. Discovering constraints after implementation wastes sessions. -->

## Scope Fence

IN: measuring the unlinked-GO population, identifying the leak mechanism in the decide flow,
proposing (not building) the structural fix, filing any vendored-code defect upstream (G-062).
OUT: building the fix (separate build task on GO), retro-linking 77 historical inceptions by hand,
changing vendored `update-task.sh` locally.

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

**Recommendation:** GO

**Rationale:** Reproduced 2026-09-20: 80/167 GO inceptions strictly unlinked and the leak is ongoing (13 new in 2026-08). BUT the loose measure collapses to 4 — this is primarily a traceability/metadata defect (related_tasks never written by the decide path, lib/inception.sh:775 prints advice only), not 77 lost approvals. GO on: (1) local check-go-propagation.sh + git-tracked baseline ledger (fires on NEW leaks only, T-2818 fatigue lesson); (2) upstream filing for a decide-time --follow-on flag (vendored, G-062); (3) surface the 4 genuine orphans (T-954, T-955, T-958, T-1698) to the human. Full evidence: docs/reports/T-3003-go-propagation-leak-investigation.md

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

### 2026-09-19T22:35:35Z — status-update [task-update-agent]
- **Change:** tags: +arc:arc-009

### 2026-09-20T08:49:07Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)
