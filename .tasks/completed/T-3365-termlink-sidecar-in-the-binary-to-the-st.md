---
id: T-3365
name: "`termlink sidecar` in the binary to the step-1 contract (OD-9)"
description: >
  Build the receive side into the termlink binary to the OD-3/4/5/7/8 contract, then
  switch per host after the two-agent acceptance test with a negative control, retiring
  AEF's receiver on that host.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [arc:arc-011]
arc_id: arc-011
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/main.rs]
related_tasks: [T-3344]
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
created: 2026-10-06T10:16:37Z
last_update: 2026-10-12T22:13:46Z
date_finished: 2026-10-12T22:13:46Z
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
cost_estimate_proposed:
  - ts: '2026-10-06T10:17:43Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius:
      tier: 2
      effort: 5
    rationale: blast_radius=? (no-components-UNMEASURED-not-zero); tier=2 
      (workflow:build); effort=5 (lines=116,acs=3)
    rubric_sha: e4a00f38e801
bvp_scores_proposed:
  - ts: '2026-10-06T10:17:59Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 2
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=0 (no-signal); 
      D4=0 (no-signal); F-RECALL=2 (body:lightly-promoted); F-ORCH=0 (no-signal)
    rubric_sha: e4a00f38e801
---

# T-3365: `termlink sidecar` in the binary to the step-1 contract (OD-9)

## Context

OD-9 ruling (log OD-9): exactly one receiver per inbox at every moment; AEF's receiver runs now where AEF is installed. Ruling summary: docs/reports/T-3344-step1-rulings-summary.md. Do not start before the step-8 plan confirms scope.

Arc: arc-011. Rulings summary: `docs/reports/T-3344-step1-rulings-summary.md`.

## Acceptance Criteria

### Agent
<!-- Scope narrowed 2026-10-12 to PH-2 row P1 (docs/design/arc-011-ph2-architecture.md; OQ-a ruled A).
     The earlier ACs moved: stages to the hub record (done in PH-1, BV-4); one receiver per inbox (store lock
     done in PH-1; hub consumer list is row P17); two-agent acceptance with negative control is row P24 / W4. -->
- [x] `termlink sidecar <verb> ...` runs every verb of the `termlink-sidecar` binary (start, hook, finalize, send, reply, status, admin) through one library entry point; the binary stays as a thin alias
- [x] The hook's detached finalizer re-executes the same binary with the right prefix (`termlink sidecar finalize` when run as `termlink`)
- [x] Production start mode: a store under a root carrying the production marker (created only by the operator) starts with the real HOME and runtime dir; the estate rule (all three inside one estate) is unchanged for estate roots; a store under no marker, or a marker of the wrong kind, is refused
- [x] Status reports the mode (estate or production)
- [x] Tests: subcommand dispatch, finalizer prefix, production marker accept/refuse cases; a mutant that accepts an unmarked store turns a test red

## Verification

cargo test -p termlink --test sidecar_subcommand
cargo test -p termlink-sidecar --test production_mode --test cli_prefix --test estate_kit --test api_tests

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

<!-- REQUIRED for arc-tagged build tasks (tags include arc:*). Captures how
     understanding evolved during build — what was learned that wasn't known at
     filing, what in the original plan no longer fits, what triggered pivots
     or new sub-tasks. Mandatory at slice boundaries (when applicable) and
     before --status work-completed.

     Origin: T-1717 grill Q4 — "the understanding of what we need and want
     evolves with the process of materialisation." Structural counter to §ACD:
     spec-vs-build divergence is logged as soon as it happens, not lost as
     folklore.

     Format (one entry per slice boundary or significant insight):
       ### YYYY-MM-DD — [topic]
       - **What changed:** [what we learned that we didn't know at filing]
       - **Plan impact:** [what in the plan no longer fits]
       - **Triggered:** [new sub-task / pivot / scope cut, with task ID if filed]

     The completion gate (T-1718) blocks --status work-completed when this
     section exists but is empty/template-only. Use --skip-evolution to bypass
     (logged Tier-2). Non-arc tasks may leave this empty.
-->

### 2026-10-13 — scope narrowed to PH-2 row P1; production mode added
- **What changed:** filed (step 1) as "the whole sidecar in the binary"; PH-1 built the sidecar as its own crate and binary, so what remained was the subcommand plus a way to run outside a test estate. Folding in exposed two traps not in the plan: `termlink` runs inside a tokio runtime and the sidecar builds its own (nested runtime panics; fixed by a plain thread), and the hook's detached finalizer re-executes `current_exe`, which under `termlink` must carry the `sidecar` prefix.
- **Plan impact:** the earlier ACs moved to their PH-2 rows (stages done in PH-1; consumer list P17; two-agent acceptance P24/W4). The production marker is written only by the operator, so row P7's runme action creates it.
- **Triggered:** none new; P7 carries the marker creation.

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

### 2026-10-06T10:16:37Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3365-termlink-sidecar-in-the-binary-to-the-st.md
- **Context:** Initial task creation

### 2026-10-12T21:49:03Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)

## Reviewer Verdict (v1.5)

- **Scan ID:** R-efb707b5
- **Timestamp:** 2026-10-12T22:13:50Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-10-12T22:13:46Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
