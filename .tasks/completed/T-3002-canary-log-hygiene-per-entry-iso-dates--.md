---
id: T-3002
name: "Canary log hygiene: per-entry ISO dates + heartbeat-freshness cross-check"
description: >
  S-28/C-39: canary logs carry entries with no per-entry ISO dates and some logs are
  34-36 days silent with fresh heartbeats; add per-entry dating and a heartbeat-vs-log
  freshness cross-check. Evidence: docs/reports/VALUE-REVIEW-repo-2026-09-19-consolidated.md
  C-39.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [value-review, arc:arc-009]
components: [tests/canary-entry-date-fixtures.sh, scripts/check-fleet-doorbell-mail-health.sh, 
      scripts/check-hook-counter-integrity.sh, 
      scripts/check-stale-waker-code-freshness.sh, 
      scripts/check-stuck-claims-freshness.sh, 
      scripts/check-waker-liveness-freshness.sh]
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
created: 2026-09-19T22:24:52Z
last_update: 2026-09-29T09:54:02Z
date_finished: 2026-09-29T09:54:02Z
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
cost_estimate_proposed:
  - ts: '2026-09-20T08:45:20Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius:
      tier: 2
      effort: 8
    rationale: blast_radius=? (no-components-UNMEASURED-not-zero); tier=2 
      (workflow:build); effort=8 (lines=207,acs=4)
    rubric_sha: e4a00f38e801
  - ts: '2026-09-29T09:51:37Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 5
      tier: 2
      effort: 7
    rationale: blast_radius=5 (5-components-medium-blast); tier=2 
      (workflow:build); effort=7 (lines=179,acs=4)
    rubric_sha: e4a00f38e801
---

# T-3002: Canary log hygiene: per-entry ISO dates + heartbeat-freshness cross-check

## Context

C-39 has two halves. Measured 2026-09-29 (T-3211 R4):
- **Heartbeat-vs-log cross-check: already delivered** by T-2842/T-2843. The heartbeat is written on completion (EXIT trap), and `scripts/canary-status.sh` classifies a log whose entries are older than a fresh heartbeat as HEALTHY (condition cleared), not FIRING. Live: stale-waker-code / stuck-claims / hook-counter / doorbell-mail read HEALTHY with non-empty logs; waker-liveness reads FIRING.
- **Per-entry ISO dates: NOT delivered.** Non-empty logs with zero dated lines: fleet-doorbell-mail (6), hook-counter-integrity (143), stale-waker-code (51), stuck-claims (112), waker-liveness (405). The convention already exists: `substrate-preflight.sh:839` frames each --quiet emission as `=== <UTC ISO> ===` … `---`.

Scope: add that frame to the --quiet firing emission of the five scripts above. Output shape without --quiet is unchanged.

## Acceptance Criteria

### Agent
- [x] Heartbeat-vs-log freshness half verified as already delivered (T-2842/T-2843; `canary-status.sh --json` statuses cited in Context)
- [x] Each of the 5 scripts, when it FIRES under `--quiet`, emits a `=== <YYYY-MM-DDTHH:MM:SSZ> ===` line before its output
- [x] Healthy `--quiet` runs still emit nothing (empty-log = healthy preserved)
- [x] Existing fixture suites for these canaries still pass
- [x] New hermetic suite `tests/canary-entry-date-fixtures.sh` pins the frame (10 assertions; 4 fail against the pre-change scripts)

## Verification

bash tests/canary-entry-date-fixtures.sh > /tmp/.t3002-v1.out 2>&1 && grep -q "10 passed, 0 failed" /tmp/.t3002-v1.out
bash tests/stuck-claims-check-fixtures.sh > /tmp/.t3002-v2.out 2>&1 && grep -q " 0 failed" /tmp/.t3002-v2.out
bash tests/hook-counter-integrity-fixtures.sh > /tmp/.t3002-v3.out 2>&1 && grep -q " 0 failed" /tmp/.t3002-v3.out
bash tests/stale-waker-code-canary.sh > /tmp/.t3002-v4.out 2>&1 && grep -q "ALL PASS" /tmp/.t3002-v4.out
bash scripts/test-check-fleet-doorbell-mail-health.sh > /tmp/.t3002-v5.out 2>&1 && grep -q "0 fail" /tmp/.t3002-v5.out
bash scripts/check-canary-log-hygiene.sh > /tmp/.t3002-v6.out 2>&1

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

### 2026-09-29 — half of C-39 had already shipped
- **What changed:** the heartbeat-vs-log cross-check was delivered by T-2842/T-2843 after the review; only the per-entry dates were missing.
- **Plan impact:** scope cut to the date frame on 5 canaries, reusing substrate-preflight's `=== <ts> ===` convention instead of inventing one.
- **Triggered:** none. Existing undated entries in the live logs are left as they are (history is not rewritten).

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

### 2026-09-19T22:24:52Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3002-canary-log-hygiene-per-entry-iso-dates--.md
- **Context:** Initial task creation

### 2026-09-19T22:35:34Z — status-update [task-update-agent]
- **Change:** tags: +arc:arc-009

### 2026-09-29T09:51:48Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

## Reviewer Verdict (v1.5)

- **Scan ID:** R-e43cb265
- **Timestamp:** 2026-09-29T09:54:07Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Reviewer:** inline
- **Findings:** none

### 2026-09-29T09:54:02Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
