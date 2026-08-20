---
id: T-2808
name: "Run the BVP estimator across active tasks and surface the HV/LC and HV/HC quadrants"
description: >
  The estimator has been unavailable all session — bvp.sh, policy/value-drivers.yaml
  and estimator.py were all untracked, so `fw bvp` failed in this worktree. T-2806/T-2807
  recovered them. Run it, produce a ranked view, and say which active work is genuinely
  high-value.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [governance, bvp, prioritisation]
components: []
related_tasks: [T-2806, T-2807, T-1918, T-1924, T-2223]
created: 2026-08-20
last_update: '2026-08-20T15:21:22Z'
date_finished:
bvp_scores_proposed:
  - ts: '2026-08-20T15:20:38Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 4
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=0 (no-signal); 
      D4=0 (no-signal); F-RECALL=0 (no-signal); F-ORCH=4 (body:rubric-routable)
    rubric_sha: e4a00f38e801
cost_estimate_proposed:
  - ts: '2026-08-20T15:21:22Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 6
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=6 
      (no-signal)
    rubric_sha: e4a00f38e801
---

# T-2808: Run the BVP estimator and surface the value quadrants

## Context

The standing instruction for this session includes *"focus on HV/LC & HV/HC tasks and run BVP
estimator regularly"*. That has not been possible: `fw bvp` failed on a missing library, because
`lib/bvp.sh`, `policy/value-drivers.yaml`, `policy/bvp-scoring-rubric.md` and
`agents/termlink/bvp-estimator/estimator.py` were all untracked and therefore absent from this
worktree. T-2806 and T-2807 recovered them, and `fw bvp` now runs.

Its first answer was `No tasks have bvp_scores: set yet.` — which is a real finding rather than
an empty result. Confirmed scores are **§ACD sovereignty-gated**: `fw bvp confirm` requires
`--i-am-human`. So an agent cannot produce confirmed scores, by design, and the ranking an
agent can legitimately produce is the **proposed** one (`--include-proposed`), which the tool
marks with a `SOURCE` column precisely so the two are never confused.

## Approach

Run the estimator and report the ranking, with the confirmed/proposed distinction carried
through into anything said about it. Where the estimator has no opinion, say so rather than
substituting my own judgement dressed as a score — an unscored task is unscored, and quietly
filling that in is exactly the sovereignty boundary the `--i-am-human` gate exists to hold.

## Scope boundary

Produces a ranking and a written read of it. Does **not** run `fw bvp confirm` on anything —
that is §ACD-gated to the human and no broad "proceed as you see fit" delegates it (CLAUDE.md
Autonomous Mode Boundaries: initiative is delegated, authority is not). Does **not** change
driver weights. Does **not** reprioritise any task's `horizon` off the back of a proposed score.

## Acceptance Criteria

### Agent
- [x] `fw bvp` runs without error in this worktree, proving the T-2806/T-2807 recovery
- [x] The ranking is produced with `--include-proposed`, and the confirmed/proposed
      distinction is preserved in what is reported
- [x] Both quadrant views (`--quadrant hv-lc`, `--quadrant hv-hc`) are run and reported
- [x] No task is scored or confirmed by the agent — the §ACD gate is left intact
- [x] The result is reported to the operator with an explicit statement of what the estimator
      does and does not currently have an opinion on

## Verification

# The estimator runs — this is what was broken for the whole session.
.agentic-framework/bin/fw bvp >/dev/null 2>&1
# Its library and policy inputs are tracked, so this survives a clean clone.
test -n "$(git ls-files .agentic-framework/lib/bvp.sh)"
test -n "$(git ls-files .agentic-framework/policy/value-drivers.yaml)"
test -n "$(git ls-files .agentic-framework/agents/termlink/bvp-estimator/estimator.py)"
# No task carries agent-set confirmed scores — the sovereignty gate held.
test -z "$(grep -rl '^bvp_scores:' .tasks/active/ 2>/dev/null)"

## Outcome

`fw bvp` runs. Starting state was 3 of 198 active tasks scored, 0 confirmed, **0 costed** — so
the HV/LC and HV/HC quadrants the session was asked to prioritise by were not merely empty, they
were **uncomputable**: a quadrant is BVP median × COST median, and there was no cost axis at all.

Both axes now exist for every `started-work` task (72):

- `fw bvp estimate all --statuses started-work` → 72 proposed score sets
- the estimator's `cost-all` subcommand → 72 cost estimates. **`fw bvp` does not surface the
  `cost-*` verbs**; they exist only on `agents/termlink/bvp-estimator/estimator.py`. That gap is
  why the cost column read `-` for everything, and it is worth a CLI task of its own.

Resulting quadrants: **32 tasks HV/LC**, **6 HV/HC**.

Top of HV/LC: T-1166 (BVP 106, cost 1.4) — retire the legacy `event.broadcast` / inbox /
`file.send-receive` primitives. Independently the same task the handover named as Suggested
First Action, which is mild corroboration that the heuristic is tracking something real rather
than keyword noise. Then T-2687 (99), T-2197 (81), T-2203 (80), T-2016 (78).

HV/HC is six tasks clustered at BVP 70 / cost ~3.7, four of them charter non-goal #4 violations.

Every score is `proposed`. **Nothing is confirmed**, and the `SOURCE` column says so on every row.

## Decisions

### 2026-08-20 — Install the scoring rubric before trusting the run, and redo the first one

- **Context:** the first pass over all 72 tasks wrote `rubric_sha: missing` on every record.
  `RUBRIC_PATH` resolves to `PROJECT_ROOT/policy/bvp-scoring-rubric.md`, and this project had
  `policy/value-drivers.yaml` but never the rubric.
- **Chose:** revert all 72 writes, install the framework's own rubric template at
  `policy/bvp-scoring-rubric.md`, re-run.
- **Why:** the rubric is provenance, not a scoring input — the scores are byte-identical either
  way — so it would have been easy to shrug at. But 72 records stamped `missing` are 72 records
  that can never be traced to the rubric version that produced them, and re-running later would
  NOT have fixed them: `write_proposed` short-circuits on `no-change-since-last` when the scores
  match, so the bad provenance would have been permanent. Cheap to fix before the commit,
  impossible after.

### 2026-08-20 — Measure whether the estimator discriminates before writing 200 files

- **Chose:** score the whole corpus in memory first and look at the distribution.
- **Why:** the three pre-existing scored tasks all tied at BVP 48, which is consistent both with
  "the heuristic is flat" and with "three is a small sample". Writing proposed scores into ~200
  task files is a large, reflow-heavy diff, and it is only worth it if the output separates the
  work. It does: BVP 0–18 raw across 72 tasks, 15 distinct values. Had it been flat, the right
  answer was to report that and write nothing.

### 2026-08-20 — Report proposed scores; never confirm them

- **Chose:** `--include-proposed` for the ranking; no `fw bvp confirm`.
- **Why:** Confirmation is §ACD sovereignty-gated behind `--i-am-human`. The session's standing
  instruction delegates initiative — which task to pick up — not authority. Producing confirmed
  scores would make the register say a human had valued this work when none had.
