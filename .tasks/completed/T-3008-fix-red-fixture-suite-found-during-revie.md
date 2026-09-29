---
id: T-3008
name: "Fix red fixture suite found during review"
description: >
  S-29/C-44: a fixture suite is red in the current tree (found by the review baseline);
  make it green or pin the defect it exposes as its own bug task. Evidence: docs/reports/VALUE-REVIEW-repo-2026-09-19-consolidated.md
  C-44.

status: work-completed
workflow_type: test
owner: agent
horizon: null
tags: [value-review, arc:arc-009]
components: [tests/cron-drift-firing-fixtures.sh]
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
created: 2026-09-19T22:30:26Z
last_update: 2026-09-29T09:45:17Z
date_finished: 2026-09-29T09:45:17Z
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
  - ts: '2026-09-20T08:45:11Z'
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
  - ts: '2026-09-29T09:43:17Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=0 (no-signal); 
      D4=0 (no-signal); F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: e4a00f38e801
cost_estimate_proposed:
  - ts: '2026-09-20T08:45:20Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius:
      tier: 1
      effort: 8
    rationale: blast_radius=? (no-components-UNMEASURED-not-zero); tier=1 
      (workflow:test); effort=8 (lines=207,acs=4)
    rubric_sha: e4a00f38e801
  - ts: '2026-09-29T09:43:17Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 1
      effort: 6
    rationale: blast_radius=1 (single-component); tier=1 (workflow:test); 
      effort=6 (lines=135,acs=4)
    rubric_sha: e4a00f38e801
---

# T-3008: Fix red fixture suite found during review

## Context

C-44 (value review 2026-09-19): `tests/cron-drift-firing-fixtures.sh` was red, root cause never investigated.

Measured 2026-09-29 (T-3211 R4):
- On this host the suite is **green** (13/13) — the red is not reproducible here.
- On GitHub CI it is **red**: Doc Lint run 36533229835 (2026-09-29 06:50Z) lists
  `FAIL fixture-suite cron-drift-firing-fixtures.sh`; the first 10 cases print PASS, the
  failing line sits in the 3 suppressed ones.
- Case 9 ("the real tree passes the firing check") runs `scripts/check-cron-install-drift.sh`
  against the **live** `/etc/cron.d` whenever that directory exists. A GitHub ubuntu runner
  has `/etc/cron.d` but none of our 31 crontabs installed. Reproduced locally with
  `CRON_DRIFT_INSTALLED_DIR=<empty dir>`: rc 1, 31 MISSING.

So the suite is not hermetic: its verdict depends on host deploy state, which contradicts the
guard-layer contract (`scripts/run-guard-layer.sh:53` — fixture suites are "hermetic by
construction"). The check's own documented rule is that a host without an install target is
informational, never firing; a CI runner is such a host, it just happens to have the directory.

## Acceptance Criteria

### Agent
- [x] Root cause identified with evidence: case 9 reads live host state (see Context)
- [x] Case 9 is skipped, with a printed reason, on a non-deploy host (`CI` set), and still runs on a deploy host
- [x] Suite green with `CI=true` and green without it on this host; the real-tree case still executes when `CI` is unset
- [x] `scripts/run-guard-layer.sh` still classifies the suite as a fixture member (no marker/name change)

## Verification

CI=true bash tests/cron-drift-firing-fixtures.sh > /tmp/.t3008-ci.out 2>&1 && grep -q "real-tree check skipped" /tmp/.t3008-ci.out
env -u CI bash tests/cron-drift-firing-fixtures.sh > /tmp/.t3008-host.out 2>&1 && grep -q "the real tree passes the firing check" /tmp/.t3008-host.out
bash scripts/run-guard-layer.sh --list > /tmp/.t3008-list.out 2>&1 && grep -q "cron-drift-firing-fixtures.sh" /tmp/.t3008-list.out

## RCA

**Symptom:** `tests/cron-drift-firing-fixtures.sh` red in the 2026-09-19 review baseline and on every GitHub Doc Lint run (e.g. 36533229835); green on the origin host.
**Root cause:** case 9 asserts the LIVE host's `/etc/cron.d` matches git whenever the directory exists. A CI runner has the directory but none of the 31 crontabs, so the check reports 31 MISSING (reproduced locally with `CRON_DRIFT_INSTALLED_DIR=<empty>`: rc 1).
**Why structurally allowed:** the guard-layer contract declares fixture suites hermetic "by convention" (`run-guard-layer.sh:53`) — nothing checks it; and CI's red was never read (0 green Doc Lint runs in the last 300).
**Prevention:** case 9 now skips with a printed reason when `CI` is set; the wider "CI guard layer red for months, nothing reads it" class is surfaced to the operator (T-3211 R4 handback, SQ-9) rather than patched here.

## Evolution

### 2026-09-29 — the red was CI-only
- **What changed:** filed as "a fixture suite is red in the current tree"; measured, it is green on the host and red only on CI, because one case reads host deploy state.
- **Plan impact:** fix is a one-branch skip, not a logic repair; the check itself was never wrong.
- **Triggered:** the same CI log shows 26 guard members red and the v0.12.0 release blocked — surfaced as SQ-9 in T-3211-R4-handback.md, not worked under this task.

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

### 2026-09-19T22:30:26Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3008-fix-red-fixture-suite-found-during-revie.md
- **Context:** Initial task creation

### 2026-09-19T22:35:36Z — status-update [task-update-agent]
- **Change:** tags: +arc:arc-009

### 2026-09-29T09:44:48Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)

## Reviewer Verdict (v1.5)

- **Scan ID:** R-0daa0ad2
- **Timestamp:** 2026-09-29T09:45:22Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Reviewer:** inline
- **Findings:** none

### 2026-09-29T09:45:17Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
