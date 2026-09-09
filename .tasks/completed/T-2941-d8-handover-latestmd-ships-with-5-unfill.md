---
id: T-2941
name: "D8: handover LATEST.md ships with 5 unfilled [TODO] sections"
description: >
  arc-008 cycle-1 audit finding. Full census: .context/audits/arc-008-cycle1-census.md

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components:
  - .context/handovers/LATEST.md
related_tasks: []
arc_id: arc-008
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
created: 2026-09-09T17:57:25Z
last_update: 2026-09-09T18:07:53Z
date_finished: 2026-09-09T18:07:53Z
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
  - ts: '2026-09-09T18:03:07Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 4
      D3: 0
      D4: 0
      F-RECALL: 1
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=4 (body:fw-audit-or-doctor); D3=0
      (no-signal); D4=0 (no-signal); F-RECALL=1 (body:episodic-only); F-ORCH=0 
      (no-signal)
    rubric_sha: e4a00f38e801
cost_estimate_proposed:
  - ts: '2026-09-09T18:03:15Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 2
      effort: 5
    rationale: blast_radius=1 (single-component); tier=2 (workflow:build); 
      effort=5 (lines=122,acs=3)
    rubric_sha: e4a00f38e801
---

# T-2941: D8: handover LATEST.md ships with 5 unfilled [TODO] sections

## Context

`fw audit` FAILs on D8: `.context/handovers/LATEST.md` ships with **5 unfilled `[TODO]` sections**.

CLAUDE.md § Session End Protocol requires filling them ("Fill in the [TODO] sections in the handover document") and § Context Budget Management is explicit that a skeleton handover is the failure mode: "Never generate a skeleton handover 'to fill in later' — the session may not survive to fill it."

An unfilled handover is not cosmetic. The handover is the entire mechanism by which the next session recovers state; a `[TODO]` where "Gotchas" should be is a silent claim that there are no traps, which is the Directive #2 shape (a plausible wrong answer rather than an error).

This is the instance; T-2942 is the structural half (10/10 recent handovers show the same shape).

## Acceptance Criteria

### Agent
- [x] All `[TODO]` markers that are actually *sections* are replaced with real content (4 of the 5: Decisions Made, Things Tried That Failed, Open Questions/Blockers, Gotchas)
- [x] The 5th occurrence is identified: it is not a section but an instructional comment emitted by the vendored generator at `handover.sh:712`, which D8 counts anyway. It cannot be filled or removed locally (G-062). Filed as **T-2943**.
- [x] Content is substantive, not a restatement of the heading — Gotchas names five concrete traps measured this session, each with the command or line number that produced it
- [x] The D8 FAIL condition (`d8_todos > 3`) is cleared: the count is now 1, verified by direct measurement against the same `grep -c` the check uses

**AC correction, recorded rather than quietly rewritten.** The first two criteria originally read "the 5 markers are replaced ... `fw audit` no longer reports D8" and asserted a count of 0. Measurement disproved the premise: D8's floor is 1 because the generator's own comment matches its counter, so a zero-count criterion is unsatisfiable and would have left this task permanently open. The criteria now assert what actually clears the FAIL (`>3`), and the residual WARN is carried by T-2943 rather than hidden here.

## Verification

```bash
c=$(grep -c '\[TODO' .context/handovers/LATEST.md || true); test "$c" -le 3
```

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

### 2026-09-09 — the target count was wrong, and the check cannot reach it

- **What changed:** The task was filed to drive `[TODO]` occurrences in LATEST.md to zero, on the assumption that all five were unfilled sections. Only four are. The fifth is an instructional comment the vendored generator emits at `handover.sh:712`, which D8 counts because it tallies raw marker occurrences across the whole file rather than in section-body position. The floor is 1, so D8's PASS branch (`audit.sh:5098`) is unreachable dead code.
- **Plan impact:** The original acceptance criteria and the `## Verification` line both asserted a count of 0. That is unsatisfiable, so as written this task could never have been closed — it would have sat in `active/` looking like unfinished work while the actual FAIL was already cleared. Both were corrected to assert the condition that genuinely clears the FAIL (`>3`), with the correction recorded in the AC block rather than silently rewritten.
- **Triggered:** T-2943 (the unreachable-PASS defect, filed as its own record per one-finding-one-task, to be reported upstream under G-062 since both files are vendored). Also revises the diagnosis in T-2942: D8b's 10/10 rate is not purely sessions failing to fill handovers, since every generated handover starts with a non-zero tally by construction — though D8b's `>3` threshold means filling still clears it.

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

### 2026-09-09T17:57:25Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2941-d8-handover-latestmd-ships-with-5-unfill.md
- **Context:** Initial task creation

### 2026-09-09T18:03:28Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-83dc25ab
- **Timestamp:** 2026-09-09T18:07:54Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-09-09T18:07:53Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
