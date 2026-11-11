---
id: T-3355
name: "arc-011 role chain step 6: pseudo-coder"
description: >
  Run step 6 (pseudo-coder) of the arc-011 design role chain (docs/design/interactive-agent-communication-role-chain.yaml);
  output docs/design/interactive-agent-communication-06-pseudo-code.md. Starts only
  after step 5 is approved by the operator.

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
created: 2026-10-06T10:13:55Z
last_update: 2026-11-11T22:11:10Z
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
  - ts: '2026-10-06T10:17:39Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 4
      effort: 7
    rationale: blast_radius=3 (2-file-refs-derived-T-3189); tier=4 
      (workflow:specification); effort=7 (lines=119,acs=5)
    rubric_sha: e4a00f38e801
bvp_scores_proposed:
  - ts: '2026-10-06T10:17:54Z'
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
  - ts: '2026-10-18T22:32:49Z'
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
  - ts: '2026-11-11T16:50:47Z'
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

# T-3355: arc-011 role chain step 6: pseudo-coder

## Context

Step 6 of the arc-011 role chain (docs/design/interactive-agent-communication-role-chain.yaml), card per the yaml. Inputs listed in the yaml for step 6. Items handed to this step by the step-1 interview: docs/reports/T-3344-step1-rulings-summary.md section 11. Iterates with step 5.

Arc: arc-011. Rulings summary: `docs/reports/T-3344-step1-rulings-summary.md`.

**Background / trigger:** Step 5 (evidence spec, T-3354) is done and its independent review is green (V-20261109-454c9bbb,
applied 2026-11-11). Operator 2026-11-11: "Proceed as you see fit and suggested" (suggestion: step 6 next).

**Problem / hypothesis:** The architecture says WHAT each component does; the evidence spec says how it will be proven,
but leaves the FAILS CLOSED WHEN conditions and nine boundary questions open for step 6 (T-3354 hand-back item 11).
Without pseudo-code the PH-2 build items (B-27 activity meter, B-16 doorbell D+, and the rest) have no agreed logic.

**Goal:** A role session (pseudo-coder card) writes the pseudo-code per component, answers step 5's questions, hands
each FAILS CLOSED WHEN back to evidence, and covers concurrency and crash cases.

**Expected outcome:** `docs/design/interactive-agent-communication-06-pseudo-code.md` reviewed independently, iterated
with step 5 where needed, ready for step 7 (review panel; the next operator approval).

## Acceptance Criteria

### Agent
- [x] Step 5 is recorded as approved by the operator before this step starts (role-chain yaml / docs/design/roles/README.md section 3) (evidence: step 5 does not decide (`decides: false`), so README 3.6 asks no operator approval of it; its independent review is green and applied (V-20261109-454c9bbb, 2026-11-11). Operator 2026-11-11, asked to tick step 5's review: "Why do I need to do the review?", and "Proceed as you see fit and suggested" with step 6 suggested. Flagged to him in chat; the wording of this criterion is template text for deciding steps)
- [x] Role session dispatched with the yaml's order of inputs (rules, common, card, adapter, profile, step facts) and a role-handback/1 record returned (evidence: inputs read in that order and hashed, 06 section 1 tables 1.1-1.3; record `docs/design/interactive-agent-communication/role-handback-step-06.json`, validated against `role-handback-1.schema.json`)
- [x] `docs/design/interactive-agent-communication-06-pseudo-code.md` exists with a version table and an inputs-of-record section citing the input hashes (evidence: sections 0 and 1, full sha256 per input; sub-documents 06a and 06b refer to the same inputs)
- [x] Pseudo-code for each component in the step-4 architecture, reconciled with step 5's evidence (evidence: 06 10.5 maps K-9a..K-34 to AL-1..AL-53; step 5's questions (a)-(i) answered in 06 section 4; every FCW names its E-n or ZR-1, register 06 section 6)
- [x] Role-chain yaml step 6 `task:` set to this task id (evidence: yaml step 6 `task: T-3355`, already present)

### Human
- [x] [REVIEW] Independent review: step 6 output meets the pseudo-coder card's completion conditions and is fit as input
      for step 7
  **Steps:**
  1. The ORCHESTRATOR (never a reviewer seat) runs `fw reviewer judge T-3355 --criterion 1` against
     `docs/design/interactive-agent-communication-06-pseudo-code.md` and its hand-back
     `docs/design/interactive-agent-communication/role-handback-step-06.json`.
  **Expected:** a green verdict bound to the reviewed revision, applied with `fw reviewer verdict apply T-3355`.
  **If not:** findings go back to the pseudo-coder role (max 2-3 rounds), then each is dispositioned.
  **Reviewer verdict:** green V-20261111-7120be32 — reviewer-judge-t-3355-r1-4dee79280791:reviewer (rung rung-1-same-vendor-independent), digest 8d67fba04ce6; dispatch judge-t-3355-r1-4dee79280791; evidence: .context/reviews/evidence/T-3355/AC1-judge-t-3355-r1-4dee79280791.md; ledger .context/reviews/verdicts.jsonl

## Verification

test -f docs/design/interactive-agent-communication-06-pseudo-code.md
grep -q 'task: T-3355' docs/design/interactive-agent-communication-role-chain.yaml

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

### 2026-10-06T10:13:55Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3355-arc-011-role-chain-step-6-pseudo-coder.md
- **Context:** Initial task creation

### 2026-11-11T16:50:46Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)

### 2026-11-11T16:51:03Z — status-update [task-update-agent]
- **Change:** horizon: now → now
