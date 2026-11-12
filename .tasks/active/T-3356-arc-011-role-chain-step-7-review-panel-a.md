---
id: T-3356
name: "arc-011 role chain step 7: review panel and disposition"
description: >
  Run step 7 (review panel and disposition) of the arc-011 design role chain (docs/design/interactive-agent-communication-role-chain.yaml);
  output docs/design/interactive-agent-communication-07-review.md. Starts only after
  step 6 is approved by the operator.

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
created: 2026-10-06T10:14:09Z
last_update: 2026-11-12T09:54:56Z
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
      effort: 7
    rationale: blast_radius=3 (2-file-refs-derived-T-3189); tier=4 
      (workflow:specification); effort=7 (lines=119,acs=5)
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
  - ts: '2026-11-11T22:15:11Z'
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

# T-3356: arc-011 role chain step 7: review panel and disposition

## Context

Step 7 of the arc-011 role chain (docs/design/interactive-agent-communication-role-chain.yaml), card per the yaml. Inputs listed in the yaml for step 7. Items handed to this step by the step-1 interview: docs/reports/T-3344-step1-rulings-summary.md section 11. 

Arc: arc-011. Rulings summary: `docs/reports/T-3344-step1-rulings-summary.md`.

**Background / trigger:** Steps 5 and 6 are done with green independent reviews (V-20261109-454c9bbb,
V-20261111-7120be32). Operator 2026-11-11 chose A for the panel (Claude Code, Codex, opencode, Antigravity).

**Problem / hypothesis:** No reviewer outside the authoring sessions has attacked the design as a whole; a build plan
(step 8) on an unattacked design builds in its holes. Build of B-27 has already started in parallel (T-3426 ruling A).

**Goal:** Per the review-panel card: one brief from the approved outputs (secrets, credentials, internal addresses
stripped), four independent seats, answers stored unedited, a synthesis with agreement and dissent, and a proposed
disposition per finding; the operator decides rejections and deferrals of security findings and approves the step.

**Expected outcome:** `docs/design/interactive-agent-communication-07-review.md` with manifest, synthesis and
disposition table; accepted findings become change requests to their steps (or to running build items).

## Acceptance Criteria

### Agent
- [x] Step 6 is recorded as approved by the operator before this step starts (role-chain yaml / docs/design/roles/README.md section 3) (evidence: step 6 does not decide (`decides: false`), README 3.6; its review is green (round 2, V-20261111-7120be32, applied 2026-11-11); operator 2026-11-11 chose the step 7 panel, which presupposes step 6 as its input)
- [x] Role session dispatched with the yaml's order of inputs (rules, common, card, adapter, profile, step facts) and a role-handback/1 record returned (evidence: docs/design/interactive-agent-communication/role-handback-step-07.json, schema-valid, bus T-3356 R-001)
- [x] `docs/design/interactive-agent-communication-07-review.md` exists with a version table and an inputs-of-record section citing the input hashes (evidence: sections 0 and 1, including the four answer files)
- [x] External review panel run on steps 1, 4, 6; every finding dispositioned (accepted / rejected with reason / deferred) (evidence: 07-review section 4: 71 raw findings -> F-1..F-51, 42 accepted / 6 rejected / 3 deferred; Codex unavailable three times, recorded as RQ-1)
- [x] Role-chain yaml step 7 `task:` set to this task id (evidence: role-chain yaml line 94 `task: T-3356`)

### Human
- [x] [REVIEW] Independent review: step 7 output meets the review-panel card's completion conditions and is fit as input
      for step 8
  **Steps:**
  1. The ORCHESTRATOR (never a reviewer seat) runs `fw reviewer judge T-3356 --criterion 1` against
     `docs/design/interactive-agent-communication-07-review.md`, the answers in `docs/reports/T-3356-panel/` and the
     hand-back `docs/design/interactive-agent-communication/role-handback-step-07.json`.
  **Expected:** a green verdict bound to the reviewed revision, applied with `fw reviewer verdict apply T-3356`.
  **If not:** findings go back to the review-panel role (max 2-3 rounds), then each is dispositioned.
  **Reviewer verdict:** green V-20261112-ea77f764 — reviewer-judge-t-3356-r1-1474da8da2eb:reviewer (rung rung-1-same-vendor-independent), digest f90036b3f4c3; dispatch judge-t-3356-r1-1474da8da2eb; evidence: .context/reviews/evidence/T-3356/AC1-judge-t-3356-r1-1474da8da2eb.md; ledger .context/reviews/verdicts.jsonl
- [ ] [REVIEW] Step 7 output approved
  **Sovereignty:** operator decision only.
  **Steps:**
  1. The agent presents, one at a time, the security findings proposed for rejection or deferral (F-7, F-36, F-37,
     F-38, F-39 reject; F-25 defer with risk acceptance RQ-3), then F-1 (a change to ruled watchdog text), RQ-2 and
     RQ-1 (close the panel at three vendors).
  **Expected:** each ruled and recorded under Decisions; then the operator approves step 7.
  **If not:** say what to change; the review-panel role revises.
  (Added 2026-11-12 by the orchestrator: README section 5 requires both criteria on every deciding step.)

## Verification

test -f docs/design/interactive-agent-communication-07-review.md
grep -q 'task: T-3356' docs/design/interactive-agent-communication-role-chain.yaml

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

### 2026-11-11 — who reviews, and may the material leave the project (operator: "A")
- **Chose:** A, a mixed panel from the operator's subscriptions: Claude Code (Anthropic), Codex (OpenAI), opencode
  (Z.ai GLM), Antigravity (Google). The brief goes to those four vendors, with secrets, credentials and internal
  addresses stripped (card 3.1-3.2). Scored A +35, C +28, B +7.
- **Rejected:** B Claude Code only (same vendor as the authors); C A plus a paid OpenRouter seat (cost, small gain).

### 2026-11-12 — five security findings proposed for rejection: F-7, F-36, F-37, F-38, F-39 (operator: "a")
- **Chose:** A, reject all five together, with the reasons of 07 section 3.4 / the disposition table: F-7 the release
  before claim is designed (a short vacancy, never two holders); F-36 no policy route exists for Tier 0, only the
  root-only admin path decides (FCW-157); F-37 the revocation head is signature-verified (06a AL-22 step 2; checked by
  the orchestrator); F-38 an owed answer holds 120 s, other mail still rings, the watchdog ignores it; F-39 unknown
  `last_human` means no fast takeover (FCW-55; checked by the orchestrator). All five from one seat (Antigravity).
- **Rejected:** B one by one; C accept them as text clarifications.
- **Note:** the reviewer read context: Antigravity run 2 quotes code absent from the material (07 3.4.a).

### 2026-11-12 — F-25 / RQ-3: a same-user process can forge human prompt origin and hold main (operator: "A")
- **Chose:** A, accept as a residual until an operator key exists (widens QR-3), with the detector of CR-5.11
  (reports suspicious human claims, e.g. from outside the hooks; never gates) and the AR-6 wording of CR-4.14.
- **Why:** the O-3 tightening still catches honest failure (a forgotten session renewing itself); a same-user (root
  on .107) attacker has worse options than holding main; the detector makes abuse visible; closes with Track K.
- **Rejected:** B block B-25 until the operator key (O-3 unbuilt for months); C accept without a detector.
- **Scored:** A +21, C +5, B −21.

### 2026-11-12 — F-1 (critical): the 300 s watchdog's ruled checks gain a typing re-check (operator: "A")
- **Chose:** A, approve CR-4.1: architecture 6.9.7.b (ruled O-2 item 2 text) adds "no client input in the last 30 s,
  re-read immediately before typing" to the watchdog's checks (kill switch, foreground harness, activity veto). The
  reasons READY and the conversation signal stay excluded are unchanged. With it: CR-6.1 (AL-6 step 5 S1 re-read for
  both `via` values, AL-8 invariant (1) corrected) and CR-5.1 (E-7 case with a mutant that drops the re-read).
- **Why:** without it a person typing at the 300 s mark gets the line plus Enter inside a half-written prompt, which
  O-2 set out to prevent; the watchdog's purpose (a stuck flag never holds mail forever) is not weakened.
- **Rejected:** B keep the ruled text (accept typing into the operator's prompt); C another wording.

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-10-06T10:14:09Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3356-arc-011-role-chain-step-7-review-panel-a.md
- **Context:** Initial task creation

### 2026-11-11T22:15:06Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)

### 2026-11-11T22:15:30Z — status-update [task-update-agent]
- **Change:** horizon: now → now
