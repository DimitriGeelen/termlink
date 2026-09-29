---
id: T-2997
name: "Human-AC staleness surface (partial-complete > threshold) + refresh G-008"
description: >
  S-23/C-21 surface half: 75 partial-complete tasks await human ACs with no ageing
  surface; add a session-start surface for partial-complete tasks older than a threshold,
  refresh G-008. Evidence: docs/reports/VALUE-REVIEW-repo-2026-09-19-consolidated.md
  C-21.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [value-review, arc:arc-009]
components: [.context/project/concerns.yaml]
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
created: 2026-09-19T22:20:18Z
last_update: 2026-09-29T09:59:17Z
date_finished: 2026-09-29T09:59:17Z
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
  - ts: '2026-09-29T09:58:43Z'
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
  - ts: '2026-09-20T08:45:20Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius:
      tier: 2
      effort: 8
    rationale: blast_radius=? (no-components-UNMEASURED-not-zero); tier=2 
      (workflow:build); effort=8 (lines=207,acs=4)
    rubric_sha: e4a00f38e801
  - ts: '2026-09-29T09:58:43Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 2
      effort: 6
    rationale: blast_radius=1 (single-component); tier=2 (workflow:build); 
      effort=6 (lines=180,acs=3)
    rubric_sha: e4a00f38e801
---

# T-2997: Human-AC staleness surface (partial-complete > threshold) + refresh G-008

## Context

C-21 (surface half). Measured 2026-09-29 (T-3211 R4): the surfaces the finding asks for already exist; what is stale is the register.
- **Session-start surface: exists.** `.context/handovers/LATEST.md` carries "Partial-Complete — awaiting human (72 tasks)" with the first 5 listed and a pointer to `fw review-queue`.
- **Ageing surface: exists.** `fw review-queue` prints an AGE column; the vendored audit's **D2** (`agents/audit/audit.sh:5069`, present since `313e9eaea` 2026-04-12) FAILs on >30d and WARNs on >14d — today "65 task(s) waiting >30d", oldest 150d.
- **Gap that remains, vendored (G-062):** the handover footer shows a count but not the oldest age. `handover.sh` is vendored, so that half belongs upstream, not here.
- **Register drift:** G-008 still reads "64 tasks … no CLI command … no digest in handovers" (2026-04-15). Items 1, 2 (count), and 4 of its `what_remains` have shipped.

Scope: refresh G-008 with the measured figures and what has shipped. No code.

## Acceptance Criteria

### Agent
- [x] Existing session-start and ageing surfaces identified with file:line evidence (Context)
- [x] G-008 in `.context/project/concerns.yaml` carries a dated 2026-09-29 measurement (72 awaiting, 65 >30d, oldest 150d) and names which `what_remains` items shipped
- [x] `concerns.yaml` still parses as YAML

## Verification

python3 -c "import yaml; d=yaml.safe_load(open('.context/project/concerns.yaml')); g=[c for c in (d if isinstance(d,list) else d.get('concerns',d.get('gaps',[]))) if c.get('id')=='G-008'][0]; m=g['measured_2026_09_29']; assert '72' in m and '65' in m and '150d' in m and 'review-queue' in m"
grep -q "D2: Human review queue" .agentic-framework/agents/audit/audit.sh
grep -q "Partial-Complete — awaiting human" .context/handovers/LATEST.md

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

### 2026-09-29 — the surface existed before the finding
- **What changed:** C-21 said "no staleness surface". Audit D2 (ageing thresholds) dates from 2026-04-12, and the handover footer and `fw review-queue` exist too. The stale artefact was G-008's text, not the tooling.
- **Plan impact:** no build. Register refreshed. The one real gap (oldest-age in the handover footer) is vendored.
- **Triggered:** none locally. The remaining gap needs an upstream filing, which is outward-facing and not done autonomously (SQ-7 shape).

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

### 2026-09-19T22:20:18Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2997-human-ac-staleness-surface-partial-compl.md
- **Context:** Initial task creation

### 2026-09-19T22:35:32Z — status-update [task-update-agent]
- **Change:** tags: +arc:arc-009

### 2026-09-29T09:58:55Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

## Reviewer Verdict (v1.5)

- **Scan ID:** R-889c4f3c
- **Timestamp:** 2026-09-29T09:59:18Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Reviewer:** inline
- **Findings:** none

### 2026-09-29T09:59:17Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
