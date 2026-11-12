---
id: T-3357
name: "arc-011 role chain step 8: planner"
description: >
  Run step 8 (planner) of the arc-011 design role chain (docs/design/interactive-agent-communication-role-chain.yaml);
  output docs/design/interactive-agent-communication-08-plan.md. Starts only after
  step 7 is approved by the operator.

status: started-work
workflow_type: specification
owner: agent
horizon: now
tags: [arc:arc-011]
arc_id: arc-011
components: []
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
created: 2026-10-06T10:14:23Z
last_update: 2026-11-12T17:03:14Z
date_finished:
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
  - ts: '2026-10-06T10:17:40Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 4
      effort: 8
    rationale: blast_radius=3 (2-file-refs-derived-T-3189); tier=4 
      (workflow:specification); effort=8 (lines=120,acs=6)
    rubric_sha: e4a00f38e801
bvp_scores_proposed:
  - ts: '2026-10-06T10:17:55Z'
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
  - ts: '2026-10-18T22:32:50Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      AS_DELIVERY_TRUTH: 0
      AS_FAILURE_SURFACED: 0
      AS_OPERATOR_SIMPLICITY: 0
      D1: 4
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 2
      F-ORCH: 0
    rationale: 'AS_DELIVERY_TRUTH=0 (L0: no signal); AS_FAILURE_SURFACED=0 (L0: no
      signal); AS_OPERATOR_SIMPLICITY=0 (L0: no signal); D1=4 (body:structural-gate);
      D2=0 (no-signal); D3=0 (no-signal); D4=0 (no-signal); F-RECALL=2 (body:lightly-promoted);
      F-ORCH=0 (no-signal)'
    rubric_sha: e4a00f38e801
  - ts: '2026-11-12T14:33:15Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      AS_DELIVERY_TRUTH: 0
      AS_FAILURE_SURFACED: 0
      AS_OPERATOR_SIMPLICITY: 0
      D1: 0
      D2: 0
      D3: 0
      D4: 0
      F-TRUST: 0
      F-SURFACE: 0
      F-CONV-RECALL: 0
    rationale: 'AS_DELIVERY_TRUTH=0 (L0: no signal); AS_FAILURE_SURFACED=0 (L0: no
      signal); AS_OPERATOR_SIMPLICITY=0 (L0: no signal); D1=0 (no-signal); D2=0 (no-signal);
      D3=0 (no-signal); D4=0 (no-signal); F-TRUST=0 (L0: no signal); F-SURFACE=0 (L0:
      no signal); F-CONV-RECALL=0 (L0: no signal)'
    rubric_sha: e4a00f38e801
    driver_set: product
---

# T-3357: arc-011 role chain step 8: planner

## Context

Step 8 of the arc-011 role chain (docs/design/interactive-agent-communication-role-chain.yaml), card per the yaml. Inputs listed in the yaml for step 8. Items handed to this step by the step-1 interview: docs/reports/T-3344-step1-rulings-summary.md section 11. 

Arc: arc-011. Rulings summary: `docs/reports/T-3344-step1-rulings-summary.md`.

**Background / trigger:** Step 7 (review panel, T-3356) was approved by the operator in chat on 2026-11-12 ("A");
T-3689 folded its change requests into documents 01-06 (04 v0.4, 06 v0.3, 05 v0.2). The operator said "next" to
start step 8 on 2026-11-12.

**Problem / hypothesis:** The design is complete but there is no committed build plan: build items exist ad hoc
(B-25, B-11, B-16, B-27 follow-ups, T-3688, T-3686) without typed dependencies, coverage of every R-n/E-n, or
deployment/rollback/decommission units.

**Goal:** A planner role session writes docs/design/interactive-agent-communication-08-plan.md per the planner card.

**Expected outcome:** The operator approves the plan (the last design approval); then work items are created
idempotently from it and the build queue (B-25 -> B-11 -> B-16) runs against it.

## Acceptance Criteria

### Agent
- [x] Step 7 is recorded as approved by the operator before this step starts (role-chain yaml / docs/design/roles/README.md section 3)
- [x] Role session dispatched with the yaml's order of inputs (rules, common, card, adapter, profile, step facts) and a role-handback/1 record returned
- [x] `docs/design/interactive-agent-communication-08-plan.md` exists with a version table and an inputs-of-record section citing the input hashes
- [x] Build plan with tasks on arc-011; the build tasks captured from the step-1 rulings (priority, main lease, sequence numbers, mail-hub check, standing tests, NTP check, canary, sidecar, hub canonical id) re-scoped or confirmed against the plan
- [x] R-41 typed work-message slice placed (OD-13)
- [x] Role-chain yaml step 8 `task:` set to this task id

### Human
- [x] [REVIEW] Independent review: step 8 plan meets the planner card's completion conditions (coverage complete,
      no cycle, every unit has acceptance and a verification check, no rubber-stamp criteria, rulings honoured)
  **Steps:**
  1. The ORCHESTRATOR (never a reviewer seat) runs `fw reviewer judge T-3357 --criterion 1` against
     `docs/design/interactive-agent-communication-08-plan.md` and its hand-back
     `docs/design/interactive-agent-communication/role-handback-step-08.json`.
  **Expected:** a green verdict bound to the reviewed revision, applied with `fw reviewer verdict apply T-3357`.
  **If not:** findings go back to the planner role (max 2-3 rounds), then each is dispositioned.
  **Reviewer verdict:** green V-20261112-ef81628a — reviewer-judge-t-3357-r1-7d6b2dfb08fc:reviewer (rung rung-1-same-vendor-independent), digest b6908d3b8c09; dispatch judge-t-3357-r1-7d6b2dfb08fc; evidence: .context/reviews/evidence/T-3357/AC1-judge-t-3357-r1-7d6b2dfb08fc.md; ledger .context/reviews/verdicts.jsonl
- [ ] [REVIEW] Step 8 plan approved (gate G-0, the last design approval; no work item exists before it)
  **Steps:**
  1. Open http://192.168.10.107:3003/review/T-3357 and read docs/design/interactive-agent-communication-08-plan.md
     (units P-n by phase, order table, D-2 order view with your decision points, explorations X-1..X-7).
  2. Check the first units match ruling A: P-1..P-5 small fixes and B-25, then B-11, then B-16.
  **Expected:** you approve; the agent then creates one task per P-n idempotently (card 3.2, 6.3).
  **If not:** say which unit, order or exploration to change; the planner revises the plan.

## Verification

test -f docs/design/interactive-agent-communication-08-plan.md
grep -q 'task: T-3357' docs/design/interactive-agent-communication-role-chain.yaml

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

### 2026-11-12 — G-0: approve the step 8 build plan (operator ruling)
- **Chose:** A — approve docs/design/interactive-agent-communication-08-plan.md v0.3 (71 units, 7 explorations,
  9 gates) with PD-1 (B-16 before the full B-14 status), PD-2 (step-1 task proposals of 7.1: T-3359 superseded;
  T-3360, T-3363, T-3366 split; T-3358, T-3362, T-3364, T-3361 confirmed) and PD-3 (P-1..P-3 before B-25) as recommended.
- **Why:** independent review green after three rounds (V-20261112-ef81628a); PD-1..PD-3 keep ruling A's order; every
  decommission, release cut, phase move and exploration keeps its own later gate.
- **Rejected:** B (ask PD-1..PD-3 separately: more rounds for choices that keep the ruled order), C (no change named),
  D (defer: build queue idle).
- Operator words in chat: "A" (after a plain-language explanation of the options). Authorises: work items per plan
  section 12; section 7 proposals applied to each task with a Decisions entry. Left open: G-1..G-8, X-1..X-7, FP-4 values.
- The operator ticks "[REVIEW] Step 8 plan approved" in Watchtower himself (/review/T-3357); the agent does not.

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

### 2026-11-12 — step 8 role session output (t3357-plan)
- **Output:** `docs/design/interactive-agent-communication-08-plan.md` v0.1 (a521d45f0); hand-back `docs/design/interactive-agent-communication/role-handback-step-08.json`.
- **Evidence for the ticked criteria:** step 7 approval recorded in this task's Context (operator "A", 2026-11-12) and T-3356; hand-back record schema-valid; plan section 1 (inputs with sha256), 7.1 (nine step-1 tasks re-scoped or confirmed; work items follow approval per card 3.2), 8 (R-41 placed: P-68 behind X-1); CHAIN already has `task: T-3357`.
- **Not done here:** no work item created (draft until the operator approves); independent review and approval are the orchestrator's and the operator's.

### 2026-10-06T10:14:23Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3357-arc-011-role-chain-step-8-planner.md
- **Context:** Initial task creation

### 2026-11-12T14:33:13Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)
