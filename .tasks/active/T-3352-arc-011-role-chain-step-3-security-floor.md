---
id: T-3352
name: "arc-011 role chain step 3: security floor and phasing"
description: >
  Run step 3 (security floor and phasing) of the arc-011 design role chain (docs/design/interactive-agent-communication-role-chain.yaml);
  output docs/design/interactive-agent-communication-03-floor-and-phasing.md. Starts
  only after step 2 is approved by the operator.

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
created: 2026-10-06T10:13:12Z
last_update: 2026-10-12T14:47:05Z
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
      (workflow:specification); effort=8 (lines=121,acs=7)
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

# T-3352: arc-011 role chain step 3: security floor and phasing

## Context

Step 3 of the arc-011 role chain (docs/design/interactive-agent-communication-role-chain.yaml), card per the yaml. Inputs listed in the yaml for step 3. Items handed to this step by the step-1 interview: docs/reports/T-3344-step1-rulings-summary.md section 11. 

Arc: arc-011. Rulings summary: `docs/reports/T-3344-step1-rulings-summary.md`.

## Acceptance Criteria

### Agent
- [ ] Step 2 is recorded as approved by the operator before this step starts (role-chain yaml / docs/design/roles/README.md section 3)
- [ ] Role session dispatched with the yaml's order of inputs (rules, common, card, adapter, profile, step facts) and a role-handback/1 record returned
- [ ] `docs/design/interactive-agent-communication-03-floor-and-phasing.md` exists with a version table and an inputs-of-record section citing the input hashes
- [ ] Every P1/P2 item from the step-1 rulings is placed in a phase, with the security floor that must hold in every phase
- [ ] GP-12 diagnosis-time bound and message-state cap set (handed to step 3 by the GP-12 ruling)
- [ ] CAND-3 staging: for each test dimension, the phase in which its subject exists and the test becomes required
- [ ] Role-chain yaml step 3 `task:` set to this task id

## Verification

test -f docs/design/interactive-agent-communication-03-floor-and-phasing.md
grep -q 'task: T-3352' docs/design/interactive-agent-communication-role-chain.yaml

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

### 2026-10-11 — OP-1 per-agent accounts: order and price (operator ruling)
- **Dialogue:** recommendation B; the operator asked for an external review first. GLM (independent, without the
  orchestrator's recommendation; docs/reports/T-3352-op1-review/glm.md) chose B with two amendments; Codex was
  blocked by its usage limit (retry scheduled 19:50). The operator asked what was needed; restated in plain terms
  (pilot in PH-3, price gate, all agents before PH-4, plus the two amendments); operator: "yes".
- **Chose:** B with GLM's two amendments:
  1. A0 pilot in PH-3 (once the C-15 Tier 0 route exists); price gate after the pilot, back to the operator with
     the measured price (H-28); A3 (every agent on its own account) before PH-4 exits; A4 (orchestrator, runme,
     cron, hub restarts without general root) in PH-6. PH-2/PH-3 run at base strength under RA-1.
  2. Amendment 1: the read-only inventory of which processes need root, and why, is a PH-0 exit item.
  3. Amendment 2: after the price gate, no new agent is started as root.
  Every account creation stays one Tier 0 event with its own digest (R-82.a (3)).
- **Rejected:** A (accounts before PH-2: the largest unknown blocks a release that does not need it), C (after
  PH-5: every capability phase adds power to root agents), D (pilot only, migration its own arc: target drifts).
- **Open:** the price gate itself; account design details (step 4); GLM's notes on underestimated costs (login per
  account, attach workflow, shared checkout, cron/canaries, re-pinning) and safety points (exact-argv helper,
  rollback keeps the account, half-migrated status never green) go to step 4.

### 2026-10-11 — Triage of the remaining step-3 questions, and 17 confirmations (operator ruling)
- **Dialogue:** after OP-1 the operator observed that a huge number of questions was being answered with no building
  work, and asked for reflection. Orchestrator counted ~70 operator decisions in arc-011 so far, 22 more queued in
  step 3, steps 4/7/8 still to decide, and building only after step 8; named the causes (a role chain built for an
  approval broker applied unchanged; every open choice presented one at a time at the same weight; the design
  process itself producing the approval fatigue OQ-14 ruled against). Proposed: triage step 3's questions into real
  choices (one at a time) and confirmations (one list, one explicit ruling, any item pullable), and start PH-0 building
  in parallel. Operator: "Okay, proceed as suggesting". PH-0 workers T-3383 and T-3395 ran. Then the confirmation
  list was presented in one message; operator: "yes, proceed as suggested" (explicit yes to the list).
- **Chose (confirmations, each as the document recommends):** OP-2 A (policy-route classes C-10, C-2a, C-6a, C-9c
  limited to grant records, no budget reset); OP-3 A (nine high-impact classes); OP-4 A (policy record format and
  defaults, `eligible_for_main` default no; 055 builds against it); OP-6 A (60 s detection to diagnosis, 5 s status
  call, 5 min drilled); OP-7 A (14 state values, 8 markers); OP-8 A (AEF's receiver: same behavioural tests, no
  size/count rule); OP-9 A (reconciliation rules RC-1..RC-9); OP-10 A (the floor: 31 invariants, 8 extra items, the
  five binding rules 3.1.b-f); OP-11 A (seven phases, basic version, CAND-3 staging); OP-13..OP-20 = RA-1..RA-8 as
  recommended in section 11 (RA-4 with a review point at the exit of PH-4; RA-7 accepted as RR-21).
- **Real choices remaining, one at a time:** OP-21 (delivery point; blocks the PH-1 basic version, first), OP-22
  (receive side vs AEF), OP-23 (RA-9, old rails unframed until PH-4), OP-12 (hypothesis thresholds incl. H-21's
  10 approval events a day), OP-5 (where the operator decides interim approvals).
- **Process change (operator-approved):** PH-0 building starts now in parallel with the remaining decisions;
  compressing chain steps 4-6 is a later process decision.

### 2026-10-11 — OP-1 additions from the Codex review, and a per-project security level (operator ruling + direction)
- **Dialogue:** Codex's OP-1 review (docs/reports/T-3352-op1-review/codex.md, after its usage limit lifted) agreed with
  B and proposed four tightenings; the orchestrator explained them in plain terms; operator: "yes, go", adding: "With
  the new ones we always need to have tier 0 override and we really have to look at the risk of the project. Not every
  project needs this [stringency] so maybe it should also be a configurable security level."
- **Chose (added to OP-1):**
  1. A3 (every agent on its own account) before PH-4 configurable authority becomes OPERATIONAL (not merely before
     PH-4 exits); PH-4 development may run alongside the migration.
  2. Before A3 counts as done, no root process (the orchestrator included) executes anything an agent can write;
     the security-critical part of A4 moves forward; routine A4 cleanup stays in PH-6.
  3. A minimal compliant account-creation helper and its approval path come WITH the A0 pilot; never manual creation.
  4. The price gate decides implementation, budget and schedule, not whether isolation happens; if the price is
     unacceptable, authority-dependent deployment waits while the plan is revised.
- **Operator direction (new, for the design steps to work out):**
  5. A Tier 0 override always remains available to the operator for these controls (audited, as SI-24).
  6. A configurable SECURITY LEVEL per project, set by the operator (signed policy record, CR-19/R-73, project scope;
     a change is a Tier 0 event): not every project carries the same risk. Orchestrator note recorded with the
     operator: some floor items stay on at every level (e.g. no silent loss, signatures); the costly ones (per-agent
     accounts, confirmation strength) are level-dependent; step 4 proposes the levels and which item belongs where,
     for the operator to rule.

### 2026-10-11 — OP-21 delivery point for normal mail (operator ruling)
- **Dialogue:** orchestrator recommended B against the document's A (A would slow mail compared with today, where
  AEF's PostToolUse mail check delivers to a busy agent within seconds). Explained plainly (next tool call vs end of
  turn). Operator: "Why don't we just [do it] as always?" Read as: keep today's behaviour = B; restated to the
  operator, open to correction.
- **Chose:** B: every message (normal and urgent) reaches a working agent at its next tool call. Consequences per
  12.3/CX-2: SI-18 (priority, urgency and trust decided at the receiver), SI-23 (kill switch) and the interrupt
  allow-list move from PH-2 into PH-1 and apply to all mail; the basic version delivers mid-turn and its estate
  needs the kill switch; R-48.e stands as written; R-63.a's "urgent mid-turn" becomes "all mid-turn, urgency sets
  priority" (a step-1 wording change, CX-2).
- **Rejected:** A (slower than today), C (needs PH-4; A or B needed anyway until then), D (another round).

### 2026-10-12 — OQ-2 message from a sender outside the roster (operator ruling)
- **Dialogue:** recommended A (withhold: store, REFUSED `not-allowed`, show, never answer). The operator objected
  that A cuts out cross-agent contact with no way to enable it, and that the sender hears nothing, so it cannot tell a
  refusal from a dead receiver. Two alternatives were played out and dropped: the receiving *agent* relaying the
  question (many agents are headless or spawned by agents and have no route to the operator), and routing through the
  orchestrator. Operator: the stranger must not carry the request, or carry a pass back ("with this you can still be
  falsified"); "the one that receives this request should service it": "I got somebody knocking on my door. It says it
  has access, but I don't have anything. What can you please decide?" The answer must come "from the source, me
  operator", and any pass a stranger presents must be validated with the operator anyway.
- **Chose (E):** (1) the receiving side holds the message (HELD `awaiting-operator`: stored, not delivered, not
  refused); (2) the receiving side's sidecar itself raises the question to the operator (needs-attention, listed by
  name at session start), naming the stranger's key and claimed agent and project, independent of whether its agent
  can talk to the operator; (3) only the operator answers, through the operator's own route (PH-1: admin
  `allow`/`deny` run via runme, logged with who/when/why; later the operator key / T-3385), never inside a message;
  `allow` adds the key and releases the held message; (4) a stranger-carried claim of approval counts for nothing (new
  probe with a kill-checked control); (5) the sender gets a knock back: "held for operator", then "admitted" or
  "refused by operator".
- **Rejected:** A (silent to the sender, no way in); B (stranger's text reaches the agent mid-turn); receiving agent or
  orchestrator as the relay (no route for headless agents, and an agent cannot decide under SI-8).
- **Limits accepted:** in PH-1 the operator sees the question at the next session start on this host (away-from-host
  reach is T-3385); "from the operator" means the admin route the operator runs, separated by convention until Track A
  and an operator key.

### 2026-10-12 — OQ-1 implementation language and home of the sidecar (operator ruling)
- **Chose:** A: Rust, in the TermLink workspace (`crates/termlink-sidecar`), as built for PH-1 (B1..B15).
- **Why:** the recommendation (OD-9 C): it lives in the binary's own workspace, so a re-vendor of the framework cannot
  delete it (G-062), and PH-1 was already built, tested and live-proven there (18 kill-checked controls, B14 PASS).
- **Rejected:** B, Python in the vendored framework tree with a later port (re-vendor loss; a rewrite of what exists).

### 2026-10-12 — PH-1 exit X6: the operator ran the demonstration (record)
- **What happened:** the operator chose to watch it live (option A) and ran `/opt/termlink/runme.sh` in their own
  terminal; action 17 (T-3424) ran `scripts/arc011-ph1-demo.sh` at 2026-10-12T14:22Z on commit 640e26231. Log
  `.context/working/runme-logs/runme-20261012T142212Z-2555818.log`: action 17 OK, runme rc=0.
- **Result:** `docs/reports/arc011-ph1-acceptance.md` Verdict PASS: X1 pass, OQ-2 E pass (a third agent held, admitted,
  delivered and answered), X3 17 of 17 floor items, X4 no production path opened, SI-25 clean, teardown clean.
  X2 (18 of 18 mutants killed) and X5 (soak) were met earlier (`arc011-ph1-fc-matrix.md`, `arc011-ph1-soak.md`).
- **So:** all PH-1 exit criteria X1..X6 are met. Moving to PH-2 is the operator's decision (5.9.6), not taken here.

### 2026-10-12 — Move arc-011 to PH-2 (operator ruling)
- **Chose:** A: PH-1 is closed, PH-2 starts. The next session plans PH-2 into build tasks the way PH-1 was planned
  (architecture note, then a build table), from step 3 section 6.2.3 (PH-2 scope and exit) and the items deferred to
  PH-2 (doorbell wave W2, reconciliation with the hub record SI-15, the daily two-agent canary H-14, install from the
  release artifact, SI-18/SI-23 completion, T-3423 review on 2026-11-12).
- **Rejected:** B (hold PH-1 and pick something else first).
- **Note:** recorded at 75% of the session's context budget; this session only writes the handover.

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-10-06T10:13:12Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3352-arc-011-role-chain-step-3-security-floor.md
- **Context:** Initial task creation

### 2026-10-11T11:26:17Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)
