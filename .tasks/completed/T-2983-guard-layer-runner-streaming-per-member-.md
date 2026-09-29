---
id: T-2983
name: "Guard-layer runner: streaming per-member JSON + member profiling + doc-budget
  fix"
description: >
  S-10/C-18: run-guard-layer.sh >600s wall with 0-byte JSON and no partial verdicts
  recoverable. Stream per-member verdicts as they complete, profile slow members,
  fix the stale '(seconds)' doc claim. Evidence: consolidated C-18.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [value-review, arc:arc-009]
components: [scripts/run-guard-layer.sh, tests/guard-layer-runner-fixtures.sh]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
# demo_target: true               # T-2286: optional — marks task as reserved for an orchestrated demo
#                                 # worker (e.g. arc-010 HM-A dispatches via mcp__fw__work_on). When set,
#                                 # `fw work-on T-XXX` refuses unless --i-am-demo-orchestrator (CLI) or
#                                 # FW_I_AM_DEMO_ORCHESTRATOR=1 (env) is passed. Prevents the parent
#                                 # session from consuming the captured→started-work transition the demo
#                                 # worker expects to drive. Origin OBS-057.
created: 2026-09-19T22:06:11Z
last_update: 2026-09-29T09:57:16Z
date_finished: 2026-09-29T09:57:16Z
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── BVP scoring fields (T-1918, arc-006). See docs/reports/T-1915-bvp-inception.md for semantics. ──
# bvp_scores:                     # confirmed per-driver scores 0-5, set by `fw bvp confirm` (T-1924).
#                                 # Sovereignty boundary — only set after human or agent confirmation.
#                                 # Shape: {D1: <int 0-5>, D2: <int 0-5>, D3: <int 0-5>, D4: <int 0-5>, [<free-driver-id>: <int>]...}
# bvp_scores_proposed:            # estimator-proposed scores (T-1922 worker). Persists when ≥2 delta
#                                 # from bvp_scores: on any driver (M3 v2-delta). Shape: list of timestamped entries.
# cost_estimate:                  # F8 composite: 0.6×blast_radius + 0.3×tier + 0.1×effort.
#                                 # Q2 fallback: T-shirt S/M/L/XL mapped to 2/4/6/8 when blast_radius is not yet computable.
bvp_scores_proposed:
  - ts: '2026-09-20T08:45:10Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 3
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=3 
      (body:component-discoverability); D4=2 (body:env-class-handled); 
      F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: e4a00f38e801
  - ts: '2026-09-29T09:55:39Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 3
      D4: 2
      F-RECALL: 2
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=3 
      (body:component-discoverability); D4=2 (body:env-class-handled); 
      F-RECALL=2 (body:lightly-promoted); F-ORCH=0 (no-signal)
    rubric_sha: e4a00f38e801
cost_estimate_proposed:
  - ts: '2026-09-20T08:45:19Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius:
      tier: 2
      effort: 8
    rationale: blast_radius=? (no-components-UNMEASURED-not-zero); tier=2 
      (workflow:build); effort=8 (lines=207,acs=4)
    rubric_sha: e4a00f38e801
  - ts: '2026-09-29T09:55:40Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 2
      effort: 7
    rationale: blast_radius=3 (2-components); tier=2 (workflow:build); effort=7 
      (lines=180,acs=4)
    rubric_sha: e4a00f38e801
---

# T-2983: Guard-layer runner: streaming per-member JSON + member profiling + doc-budget fix

## Context

C-18 asked for three things. Measured 2026-09-29 (T-3211 R4):
- **Member profiling: delivered** by T-3090. Every member is timed (`elapsed_s` in `--json`, seconds in human output); report `docs/reports/T-3090-guard-layer-timing.md`.
- **Stale "(seconds)" doc claim: delivered** by T-3090. CLAUDE.md now reads "minutes, not seconds — ~16min contended-host".
- **Streaming per-member verdicts: NOT delivered.** `--json` is assembled after the last member (`run-guard-layer.sh` report block), so a run killed at minute 15 of 16 leaves 0 bytes and no recoverable verdict. That is the procAsFit/CI shape: long-running, and killed by timeouts.

Scope: an additive `--jsonl PATH` that truncates PATH at start and appends one JSON object per member **as it completes** (`name, kind, rc, verdict, elapsed_s, output`). Composes with human or `--json` output; changes neither.

## Acceptance Criteria

### Agent
- [x] Profiling + doc-claim halves verified as delivered by T-3090 (cited in Context)
- [x] `--jsonl PATH` writes one parseable JSON line per member, appended as each completes, carrying name/kind/rc/verdict/elapsed_s/output
- [x] A run killed mid-layer leaves the completed members' lines in PATH (fixture kills the runner after the first member)
- [x] Existing `tests/guard-layer-runner-fixtures.sh` assertions still pass; `--json` / human output unchanged

## Verification

bash tests/guard-layer-runner-fixtures.sh > /tmp/.t2983-v1.out 2>&1 && grep -q "57 passed, 0 failed" /tmp/.t2983-v1.out
grep -q "killed run: --jsonl still holds the finished member (PASS)" /tmp/.t2983-v1.out
bash scripts/run-guard-layer.sh --help > /tmp/.t2983-v2.out 2>&1 && grep -q -- "--jsonl PATH" /tmp/.t2983-v2.out
bash -n scripts/run-guard-layer.sh

## RCA

<!-- REQUIRED for bug-class tasks (workflow_type=build with bug-tag, OR title matches
     fix/bug/rca/broken/crash/error/regression/fail/hotfix).
     Non-bug-class tasks may leave this section empty or remove it.

     For bug-class, fill in:
       **Symptom:** what was observed (the user-facing manifestation).
       **Root cause:** the specific structural/logical gap — not "the code was wrong".
       **Why structurally allowed:** what in the framework/code/tooling let this go undetected.
       **Prevention:** what catches the next instance (test/lint/gate/doc/learning) — distinct from the fix itself.

     The completion gate (T-1550, G-019) blocks --status work-completed when
     bug-class AND this section is empty/template-only. Use --skip-rca to bypass (logged).
-->

## Evolution

### 2026-09-29 — two of three asks had shipped
- **What changed:** T-3090 had already delivered profiling and the doc-budget fix; only streaming was open.
- **Plan impact:** implemented as an additive `--jsonl PATH` rather than changing `--json`'s shape (that would break its consumers). It also keeps each member's full output, which CI's 7-line human view hides (T-3216 hit exactly that on planted-default-gate).
- **Triggered:** none filed. Using `--jsonl` in CI would recover the hidden output, but CI workflow changes sit under SQ-9 (T-3216).

## Recommendation

<!-- T-2945: same shape as inception.md's block — the gate that reads it
     (audit_inception_recommendation, lib/task-audit.sh:117) is shared, so the
     shape is copied rather than reinvented.

     REQUIRED once this task reaches partial-complete: Agent ACs done, at least
     one `### Human` AC still unticked. `lib/review.sh:205-211` (T-2421) BLOCKS
     `fw task review` emission for build/refactor/test/decommission tasks in that
     state with no substantive block here — the operator would otherwise open
     /review/<id> to a blank Recommendation card and be asked to approve a form.

     Not required while every Human AC is ticked or the task has none: the gate
     only fires on the partial-complete transition. It is here from the start so
     you write it while you still have the evidence, not when the gate refuses.

     Format (the parser wants the `**Recommendation:**` line at the start of a
     line; a leading `-` or `*` bullet is also accepted):
     **Recommendation:** GO / NO-GO / DEFER
     **Rationale:** Why (cite evidence — what shipped, what was proven, what remains)
     **Evidence:**
     - Finding 1
     - Finding 2

     DEFER is for evidence gaps, not confidence gaps (CLAUDE.md §Presenting Work
     for Human Review). If the artefact is complete and you still don't want to
     commit, that is a calibration failure — recommend GO or NO-GO.
-->

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

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-09-19T22:06:11Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2983-guard-layer-runner-streaming-per-member-.md
- **Context:** Initial task creation

### 2026-09-19T22:08:36Z — status-update [task-update-agent]
- **Change:** tags: +arc:arc-009

### 2026-09-29T09:55:57Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

## Reviewer Verdict (v1.5)

- **Scan ID:** R-abb25816
- **Timestamp:** 2026-09-29T09:57:25Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Reviewer:** inline
- **Findings:** none

### 2026-09-29T09:57:16Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
