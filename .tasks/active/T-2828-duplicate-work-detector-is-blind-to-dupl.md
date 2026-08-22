---
id: T-2828
name: "Duplicate-work detector is blind to duplicated FIXES — axis C sees only added
  files and excludes main"
description: >
  Inception: Duplicate-work detector is blind to duplicated FIXES — axis C sees only
  added files and excludes main

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-08-22T10:13:27Z
last_update: '2026-08-22T10:15:20Z'
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
  - ts: '2026-08-22T10:15:00Z'
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
  - ts: '2026-08-22T10:15:20Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 4
      effort: 7
    rationale: blast_radius=3 (no-signal); tier=4 (no-signal); effort=7 
      (no-signal)
    rubric_sha: e4a00f38e801
---

# T-2828: Duplicate-work detector is blind to duplicated FIXES — axis C sees only added files and excludes main

## Problem Statement

<!-- What problem are we exploring? For whom? Why now? -->

## Assumptions

<!-- Key assumptions to test. Register with: fw assumption add "Statement" --task T-XXX -->

## Open Questions

- **IW-1: Is there a signal for "these two branches fixed the same thing" that is
  materially quieter than "these two branches touched the same file"?**
  confidence: 1
  disposition:
  rationale:

  The file is far too coarse — nearly every pair of long-lived branches touches some file in
  common, so an axis at that grain fires constantly and gets switched off within a week,
  which is worse than no axis. Candidate finer grains to evaluate: same enclosing function
  changed on both sides; overlapping hunk ranges; both sides deleting the same line; both
  sides introducing the same new identifier. Each needs measuring against this repo's real
  branch set for how often it would have fired, not just whether it catches T-2687/T-2824.

- **IW-2: Should main be compared at all, and if so as what?**
  confidence: 2
  disposition:
  rationale:

  Axis C excludes BASE by construction (`if b != BASE`). That is correct for axis C as
  written — `git diff --diff-filter=A BASE...main` is empty by definition — so including main
  in the existing branch list changes nothing. Comparing against main means a genuinely
  different comparison: each branch's changes since the merge base versus **main's** changes
  since that same base. Worth confirming that is the intended shape before building it, and
  that the merge base is per-branch rather than global.

- **IW-3: Is detection the right intervention, or is the timing wrong?**
  confidence: 1
  disposition:
  rationale:

  A check that fires at merge time reports work already wasted; T-2824's effort was spent
  eight days before anything could have noticed. The cheap manual mitigation
  (`git log origin/main -- <path>` before starting a fix) acts at the moment the cost is
  still avoidable. A pre-edit hook on that path might beat a post-hoc scan outright, and if
  so the honest outcome of this inception is NO-GO on the axis plus a hook, not a new axis.

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

**Recommendation:** GO

**Rationale:**

Two confirmed instances of the same class in this repo. T-2687 (main) and T-2824 (this branch) independently fixed the identical termlink_topics silent-partial-inventory defect eight days apart; it surfaced only as a merge conflict. Earlier, two branches independently wrote check-verification-pipefail.sh. check-task-id-collisions.sh axis C exists precisely to catch this and saw neither reliably: it runs git diff --diff-filter=A so it examines only ADDED files (a fix to an existing file is invisible), and its branch set excludes BASE so duplication against main — where 230 commits landed while this branch was open — is invisible from both directions. The catch on the first instance was luck about the shape (a new file on two feature branches), not coverage. GO rather than DEFER because the evidence is already in hand and the open question is purely design: what signal identifies duplicated fixes without firing on every pair of branches that touch a common file, which is nearly all of them. That threshold question is the whole value of the inception, and if no low-noise signal exists the honest outcome is NO-GO plus a documented manual mitigation — which is itself worth knowing.

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

### 2026-08-22T10:13:46Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
