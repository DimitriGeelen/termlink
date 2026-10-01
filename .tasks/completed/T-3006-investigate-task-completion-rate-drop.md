---
id: T-3006
name: "Investigate task completion-rate drop"
description: >
  S-29/C-42: measured completion-rate drop in the recent window; separate cause (larger
  tasks vs gate friction vs abandonment) before any process change. Evidence: docs/reports/VALUE-REVIEW-repo-2026-09-19-consolidated.md
  C-42.

status: started-work
workflow_type: inception
owner: agent
horizon: now
tags: [value-review, arc:arc-009]
components: []
related_tasks: []
created: 2026-09-19T22:28:29Z
last_update: '2026-09-22T14:57:32Z'
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
  - ts: '2026-09-22T14:57:32Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 4
      effort: 7
    rationale: blast_radius=3 (target_blast_radius:inception-T-2189); tier=4 
      (workflow:inception); effort=7 (lines=156,acs=4)
    rubric_sha: e4a00f38e801
---

# T-3006: Investigate task completion-rate drop

## Problem Statement

C-42 (run4 I3): task completions fell ~8× (609/mo → ~75/mo) while commit volume held at ~447/30d.
Before any process change, the cause must be separated: bigger tasks (same work, fewer closes) vs
gate friction (work done, close blocked — e.g. partial-complete accumulation in the human review
queue) vs genuine abandonment. For: the operator (process decisions) and calibration (BVP cost
axis). Why now: arc-009 S-29d slice; a process change made on the wrong cause is pure regression
risk.

## Assumptions

- A-1: `date_finished` in `.tasks/completed/` is reliable enough post-T-2203 repair to bucket
  completions by month.
- A-2: Commit volume per month with task-ID attribution is a usable proxy for work volume.
- A-3: The 75-task partial-complete review queue (57 >30d, per D2/T-2940) is large enough to
  account for a material share of the drop.

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

- **IW-1: Does the ~8× completion drop reproduce from `date_finished` monthly buckets, and over which window?**
  confidence: 3
  disposition: answered
  rationale: Yes from the Apr peak (609 → ~77/mo Sep run-rate); but commits fell 2114 → ~310/mo in the same window (~6.8×, near-proportional) — C-42's "commits held" was a window artifact (artifact F1)

- **IW-2: How much of the drop is explained by gate friction — tasks finished agent-side but parked in the review queue / partial-complete instead of reaching completed/?**
  confidence: 2
  disposition: answered
  rationale: A stock of 29-75 partial-completes accumulated over months — ≤1.5 months of Sep-rate completions total, not an 8× monthly flow driver (artifact F3)

- **IW-3: Did task size grow — commits-per-completed-task and distinct-task-IDs-per-month trends?**
  confidence: 3
  disposition: answered
  rationale: 1.9 (Mar) → 4.0 (Sep) across the span, but only 3.5 → 4.0 within the drop window — explains the residual, not the drop (artifact F2)

- **IW-4: Is there evidence of abandonment (tasks started-work with no commit activity for >30d)?**
  confidence: 3
  disposition: answered
  rationale: Exactly 1 stale started-work task (T-2486, itself an inception awaiting review) — negligible (artifact F4)

## Exploration Plan

1. Bucket `.tasks/completed/` by `date_finished` month; bucket commits and distinct task IDs in
   commit subjects by month (git log). Time-box: 25 min.
2. Count agent-side-done-but-not-completed population (partial-complete/review queue) and its
   entry-date distribution. Time-box: 15 min.
3. Compute commits-per-completed-task by month (size proxy). Time-box: 10 min.
4. Recommendation separating cause shares. Time-box: 15 min.

## Technical Constraints

<!-- What platform, browser, network, or hardware constraints apply?
     For web apps: HTTPS requirements, browser API restrictions, CORS, device support.
     For hardware APIs (mic, camera, GPS, Bluetooth): access requirements, permissions model.
     For infrastructure: network topology, firewall rules, latency bounds.
     Fill this BEFORE building. Discovering constraints after implementation wastes sessions. -->

## Scope Fence

IN: measuring completion/commit/queue trends from the local ledger + git history; attributing the
drop across the three candidate causes; recommendation only.
OUT: any process change, review-queue drainage (that is T-2940/D2's territory), retro-editing task
records.

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

**Recommendation:** NO-GO

**Rationale:** C-42's premise is falsified by measurement: completions and commits fell TOGETHER (Apr 609→~77/mo vs commits 2,114→~310/mo, ~6.8× near-proportional — "commits held at 447/30d" compared the current 30d window against the April completion peak without checking April's 2,114 commits). Attribution: ~85-90% total-throughput decline, ~10% task-size growth (commits/completion 3.5→4.0 in the window), small review-queue residual (owned by T-2940/D2), ~0% abandonment (1 stale task). No gate/process change is warranted. Optional cheap follow-on only: a completions-vs-commits monthly panel in fw metrics. Full series: docs/reports/T-3006-completion-rate-drop-investigation.md

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

### 2026-09-19T22:35:36Z — status-update [task-update-agent]
- **Change:** tags: +arc:arc-009

### 2026-09-20T08:52:51Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)
