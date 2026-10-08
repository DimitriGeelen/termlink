---
id: T-3351
name: "arc-011 role chain step 2: threat modeler"
description: >
  Run step 2 (threat modeler) of the arc-011 design role chain (docs/design/interactive-agent-communication-role-chain.yaml);
  output docs/design/interactive-agent-communication-02-threat-model.md. Starts only
  after step 1 is approved by the operator.

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
created: 2026-10-06T10:12:58Z
last_update: 2026-10-08T08:15:46Z
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
  - ts: '2026-10-06T10:17:38Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 4
      effort: 8
    rationale: blast_radius=3 (2-file-refs-derived-T-3189); tier=4 
      (workflow:specification); effort=8 (lines=129,acs=8)
    rubric_sha: e4a00f38e801
bvp_scores_proposed:
  - ts: '2026-10-06T10:17:53Z'
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

# T-3351: arc-011 role chain step 2: threat modeler

## Context

Step 2 of the arc-011 role chain (docs/design/interactive-agent-communication-role-chain.yaml), card per the yaml. Inputs listed in the yaml for step 2. Items handed to this step by the step-1 interview: docs/reports/T-3344-step1-rulings-summary.md section 11. Operator accepts residual risks (injection into a busy session, hub-to-hub trust, per-circuit credentials).

Arc: arc-011. Rulings summary: `docs/reports/T-3344-step1-rulings-summary.md`.

## Acceptance Criteria

### Agent
- [x] Step 1 is recorded as approved by the operator before this step starts (role-chain yaml / docs/design/roles/README.md section 3)
- [x] Role session dispatched with the yaml's order of inputs (rules, common, card, adapter, profile, step facts) and a role-handback/1 record returned
- [x] `docs/design/interactive-agent-communication-02-threat-model.md` exists with a version table and an inputs-of-record section citing the input hashes
- [x] Threat model covers adversaries ADV-1..ADV-10 with privileges (GP-15), and GP-1, GP-6, GP-13, GP-14
- [x] Covers the circuit trust model and per-circuit credential lifetime (OD-1, CAND-16), same-id/different-content (CAND-2) and same-sequence-number/different-content (CAND-18), peer-content framing (CAND-1)
- [x] Residual risks listed for the operator to accept, one per line
- [x] Role-chain yaml step 2 `task:` set to this task id

### Human
- [ ] [REVIEW] Accept the step-2 residual risks
  **Steps:**
  1. Read the residual-risk list in docs/design/interactive-agent-communication-02-threat-model.md
  2. Accept each, or ask for a mitigation
  **Expected:** Every residual risk accepted or sent back
  **If not:** Name the risk to rework

## Verification

test -f docs/design/interactive-agent-communication-02-threat-model.md
grep -q 'task: T-3351' docs/design/interactive-agent-communication-role-chain.yaml

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

### 2026-10-07 — How to take step 2's 54 operator items (operator ruling)
- **Context:** threat model v0.3 (after review rounds 1 and 2 and two revisions; GLM r2 "fit", Codex r2 "not fit yet",
  every r2 finding then disposed in disposition-r2.md; neither reviewer has seen v0.3) carries OQ-1..17, RR-1..20,
  CR-1..17.
- **Chose:** B — the 17 open questions one at a time; each brief names the change requests and residual risks its
  ruling settles; the unlinked remainder afterwards, one at a time.
- **Why:** keeps one decision per message while cutting ~54 rulings to ~17 plus a tail. Scored B +42, C +31, A +1, D −8.
- **Rejected:** A (54 rounds, many restating earlier answers), C (batches risk acceptance, against the standing rule),
  D (risk acceptance is reserved to the operator by the role card).

### 2026-10-07 — OQ-1 which added adversaries to confirm (operator ruling)
- **Chose:** A — confirm ADV-11 (compromised or rogue hub), ADV-12 (unadmitted joiner), ADV-13 (network attacker:
  delay, drop, replay, redirect; TLS gives secrecy and integrity, not delivery), ADV-14 (holder of the operator's
  approval device or key). Steps 3-4 defend against ADV-1..ADV-14.
- **Why:** each acts where the new parts sit (circuits, card exchange, approvals); the record shows hubs going wrong
  (stray hub 2026-10-04, G-060); both reviewers asked for ADV-14. Scored A +37, B +26, D −21, C −32.
- **Rejected:** B (drops ADV-13; "covered by TLS" misreads TLS, Codex r1), C (leaves CAND-17 and the approval device
  undefended), D (step 3 would not know whom the floor stops).
- **Settles directly:** no RR or CR; scopes RR-4, RR-14, RR-16, RR-18 and CR-14.

### 2026-10-07 — OQ-2 the shared operating-system user (operator ruling)
- **Fact added at the brief:** all 19 Claude agents on .107 run as root (ps, 2026-10-07), so today the shared user is
  root and same-user equals host root in practice.
- **Chose:** C now, B as the committed target (operator's words: "we accept it for now, but we definitely want to move
  to a more isolated per-agent account, which can be created by the orchestrator … always via Tier 0"). (1) RR-2 is
  accepted for the present, with the root fact. (2) Target: one isolated OS account per agent, created by the
  orchestrator through a narrow privileged helper (least privilege: one helper with fixed settings, e.g. a single
  sudoers entry; never general root for the orchestrator); every creation is a Tier 0 action (human-approved, logged).
  (3) Step 3 phases and prices the move, including group-based access to shared checkouts; step 4 designs the helper.
  (4) Host root stays residual for the lifetime (RR-2).
- **Why:** one account per workload is the standard pattern (service users, systemd DynamicUser, k8s service accounts);
  it makes per-agent keys a real boundary and ends agents running as root. Scored C +29, A +10, B (as an immediate
  requirement) +5, D −12.
- **Rejected:** A (closes the door on separation), B now (unmeasured change blocking the floor), D (step 3 needs it).
- **Settles:** RR-2 accepted for now (with the target); RR-7 bounded by it.

### 2026-10-07 — OQ-3 the circuit credential lifetime (operator ruling)
- **Evidence added at the brief:** the .107 hub journal (from 2026-10-04 only) shows 2 restarts in 3 days, down
  4m35s and ~1s; consistent with H-1 but thin.
- **Operator question answered first:** how is a sender authenticated if a credential can be stolen? Answer recorded:
  identity is the agent key pair (public key on the home hub's card, PR-1a); set-up is signed (7.2.a); the credential
  is sender-constrained and channel-bound (fresh proof of possession, 7.2.c); every message is signed (PR-1, CR-2).
  A stolen credential alone is useless; the remaining exposures are key theft (RR-2, OQ-2 target), a lying hub at
  first contact (RR-14, ADV-11, OQ-8), and no operator key yet (GP-11, OQ-6).
- **Chose:** D — PN-1 1 hour, PN-2 renewal from 30 minutes and 24 hours absolute, PN-13 1 hour, and the R-7.e (1)
  cross-host bound of 1 hour, as STARTING values; re-checked against 30 days of hub outage data (follow-up T-3383).
- **Why:** survives the measured outages with margin; a thief also needs the agent key; numbers stay open to evidence.
  Scored D +42, B +26, C +12, A −21.
- **Rejected:** A (a measured 4.5-minute restart takes a third of it), C (a working afternoon for a key thief),
  B without a dated re-check (rests on 3 days of one hub).
- **Settles:** PN-1, PN-2, PN-13, the R-7.e (1) bound; RR-5 accepted at a one-hour window.

### 2026-10-08 — OQ-4 same-host new conversation without the hub (J4 question; operator ruling)
- **Chose:** D — A now: a new conversation always goes through the hub, on one host too (R-7.e (2) unchanged); only
  established conversations run on a circuit. Revisit B (local start while the hub is down) once per-agent OS
  accounts exist (OQ-2 target) and T-3383 has measured hub-down time.
- **Why:** a local start would be a second authority (R-45, R-49); with every agent running as root, "same user"
  cannot tell agents apart (7.6.b); the OQ-2 target removes that obstacle, so B gets a second look then. Scored
  D +42, A +33, B +9, C −17.
- **Rejected:** B now (trusts anyone on the host), C (bypasses the hub's checks permanently), A with no revisit.
- **Settles:** RR-11 accepted for now; TB-6 analysis (14.7, written for A) stands.

### 2026-10-08 — OQ-5 failures the store must survive (GP-13; operator ruling)
- **Chose:** A — survive F-1 (sidecar killed mid-write) and F-2 (host crash: flush before STORED); detect and show
  F-3 (disk full: refuse STORED visibly), F-4 (corrupt record: checksum, quarantine, resend by digest), F-5 (store lost
  or old copy restored: reconcile with the hub record, flag lost-after-STORED); refuse or detect F-6 (second writer,
  lock) and F-7 (attacker writes; prevention is RR-2).
- **Why:** STORED releases the sender, so it must hold through a power cut; every other failure is loud, never
  silent. Scored A +42, C +32, B −18.
- **Rejected:** B (no flush; breaks the STORED promise in a crash for unmeasured speed), C (duplicate writes for a
  failure the resend path already recovers).
- **Settles:** PR-17, SI-15, SI-16 to step 3 as the durability floor. Leaves recovery after F-5 to OQ-10 / CR-9.

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-10-06T10:12:58Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3351-arc-011-role-chain-step-2-threat-modeler.md
- **Context:** Initial task creation

### 2026-10-06T23:07:28Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)
