---
id: T-3353
name: "arc-011 role chain step 4: architect"
description: >
  Run step 4 (architect) of the arc-011 design role chain (docs/design/interactive-agent-communication-role-chain.yaml);
  output docs/design/interactive-agent-communication-04-architecture.md. Starts only
  after step 3 is approved by the operator.

status: started-work
workflow_type: design
owner: agent
horizon: now
tags: [arc:arc-011, security]
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
created: 2026-10-06T10:13:26Z
last_update: 2026-11-03T16:11:06Z
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
      tier: 3
      effort: 8
    rationale: blast_radius=3 (2-file-refs-derived-T-3189); tier=3 
      (workflow:design); effort=8 (lines=120,acs=6)
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
  - ts: '2026-10-18T22:32:47Z'
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
  - ts: '2026-11-02T21:47:34Z'
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

# T-3353: arc-011 role chain step 4: architect

## Context

**Background / trigger:** operator 2026-11-02 ("yes, 1 and 2"): the remaining arc-011 work that sets a security
boundary, an interface others build on, or spans hosts goes through the role sequence; steps 1-3 are done (step 3's
confirmations ruled 2026-10-11, e12bb7298); the main lease (R-67) was built and rolled out today (T-3654/T-3660).

**Problem / hypothesis:** the remaining PH-2 rows (P10 hub signing key, P11 revocation list, P13, P15 blobs, P16-P19,
P21, P22, P24) and R-67.a1(4) forward-once have no architecture and no tasks, so they cannot be built safely.

**Goal:** the step-4 architecture document per the architect card, with a build table mapping each item to a task.

**Expected outcome:** `docs/design/interactive-agent-communication-04-architecture.md` + role hand-back, ready for the
independent review; only real decisions go to the operator.

Step 4 of the arc-011 role chain (docs/design/interactive-agent-communication-role-chain.yaml), card per the yaml. Inputs listed in the yaml for step 4. Items handed to this step by the step-1 interview: docs/reports/T-3344-step1-rulings-summary.md section 11. 

Arc: arc-011. Rulings summary: `docs/reports/T-3344-step1-rulings-summary.md`.

## Acceptance Criteria

### Agent
- [x] Step 3 is recorded as approved by the operator before this step starts (role-chain yaml / docs/design/roles/README.md section 3) (evidence: e12bb7298 2026-10-11 records the operator's step-3 confirmations incl. OP-10 A floor and OP-11 A phasing; later rulings f0e99b302 and d44495678 the same day; T-3352 work-completed; operator's go for step 4 2026-11-02 "yes, 1 and 2" (f1ca9f841). Note: the chain yaml's step-3 entry has no `approval:` line yet, unlike steps 1 and 2; the orchestrator may add it)
- [x] Role session dispatched with the yaml's order of inputs (rules, common, card, adapter, profile, step facts) and a role-handback/1 record returned (evidence: docs/design/interactive-agent-communication/role-handback-step-04.json, validated against schemas/role-handback-1.schema.json; inputs read in that order, hashes in the output §1)
- [x] `docs/design/interactive-agent-communication-04-architecture.md` exists with a version table and an inputs-of-record section citing the input hashes (evidence: §0 version table, §1 inputs of record with sha256; sub-document 04a-traceability)
- [x] Every value left open to step 4 is decided or listed as open (summary section 11 row 4: hub-id minting, grant format, retention ceilings, jitter/cap/hop limit, skew bound, gap-wait bound, mail-hub declaration format, idle no-surfacing threshold, 'main' lease durations and priority format, GP-4 offline) (evidence: §8 table V-1..V-16 incl. the J5 numbers V-11..V-13; GP-4 in §8.2; GP-11 listed open as V-16)
- [x] Ownership split TermLink/AEF stated per component (OD-7, OD-9, CAND-10) (evidence: §16 table; owner column in §5.1)
- [x] Role-chain yaml step 4 `task:` set to this task id (evidence: role-chain yaml step 4 `task: T-3353`, already present)

### Human
- [ ] [REVIEW] Independent review: step 4 output meets the architect card's completion conditions and is fit as input
      for step 5
  **Steps:**
  1. The ORCHESTRATOR (never a reviewer seat) runs `fw reviewer judge T-3353 --criterion 1` against
     `docs/design/interactive-agent-communication-04-architecture.md` and its hand-back
     `docs/design/interactive-agent-communication/role-handback-step-04.json`.
  **Expected:** a green verdict bound to the reviewed revision, applied with `fw reviewer verdict apply T-3353`.
  **If not:** findings go back to the architect role (max 2-3 rounds), then each is dispositioned (accept with change
  request / reject with reason / defer with risk acceptance).
- [ ] [REVIEW] Step 4 output approved
  **Sovereignty:** operator decision only.
  **Steps:**
  1. Read the operator questions O-1..O-7 (section of the architecture doc), presented one at a time by the agent.
  **Expected:** each ruled and recorded under Decisions; residual risks and change requests XR-1..XR-6 decided.
  **If not:** say what to change; the architect role revises.

## Verification

test -f docs/design/interactive-agent-communication-04-architecture.md
grep -q 'task: T-3353' docs/design/interactive-agent-communication-role-chain.yaml

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

### 2026-11-03 — step 4 independent review: accept two green verdicts (operator ruling A)
- **Chose:** A (operator: "A"). Round 2 (rung 5, architecture v0.2) is accepted as the independent review on the green
  verdicts of two vendors: V-20261103-1fb14fee (claude-code) and V-20261103-a97bbeb8 (antigravity), no findings.
- **Why:** the other seats failed for tooling reasons, not content: codex not-evaluated (cannot launch Chromium to
  render Mermaid), opencode no verdict (its render tmp dir auto-rejected as external), both AEF T-5042; the panel's
  fall-through to a paid seat cannot complete a run today (AEF T-5046, T-5043). Round 1 (V-20261103-717b2df9, amber)
  findings were all fixed in v0.2.
- **Rejected:** B paid seat (approved paid seats are never dispatched into the panel today, AEF T-5046); C wait for
  AEF's fixes (delays the operator's O-1..O-7 rulings for a tooling defect).
- **Left open:** the "[REVIEW] Independent review" Human box is the operator's to tick (agent never ticks Human ACs);
  next: operator questions O-1..O-7 one at a time.

### 2026-11-03 — O-1 cross-hub file delivery: A, encrypted only, for now (operator ruling)
- **Chose:** A (operator: "For now yes, but we want to improve it for the future"). `artifact put --hub` only with
  `--encrypt-to <agent>`; a plaintext cross-hub put is refused with a reason (DR-5 A, 6.4.6).
- **Why:** the only option where a file's content is not readable on another machine's hub; costs ring20 a short wait
  for per-agent keys (B-11). Scored A +16, B -7, C -16 (product set).
- **Rejected:** B plaintext between the operator's hubs with a risk acceptance; C defer to PH-4.
- **Left open (operator's "improve for the future"):** existing routes that already carry files between machines are
  NOT covered by O-1 and stay as they are for now: `termlink file send` (TLS in transit, readable on the hub), the runme
  fleet upgrade stream, ssh/scp, Pen's SMB mounts, 055's ssh gate. Follow-up: bring them under the same rule later.

### 2026-11-03 — O-2 doorbell: D+ with an external 300 s fallback (operator ruling; supersedes DR-6 A)
- **Chose:** D+ (operator: "D plus is fine but we also need to have a fallback option ..."). Not A: the operator
  rejected A because agents must receive and act on messages without a person pressing Enter.
  1. Normal ring: only when the session's adapter READY flag says the agent has finished its turn, AND no one typed in
     that terminal recently (activity veto, T-3387; 30 s), AND no conversation is under way in that session.
  2. Fallback (outside the agent's own flag, a watchdog the sidecar runs): if NOTHING has happened in the session for
     300 s, inject anyway even when the flag says busy, so a stuck or false flag can never hold a message forever.
     "Nothing happened" = no typing AND no human-to-agent or agent-to-agent communication in either direction (operator:
     "that means typing but also H2H communication, so you need to figure out how you can measure that").
  3. Delivery stays strict: a typed line is ATTEMPTED; HANDED_OVER only from transcript evidence (as built in AEF
     inject.py/hooks.py: "Injection alone never records HANDED_OVER"; no evidence in 90 s = HANDOVER_UNCONFIRMED).
  4. Line format without brackets (SI-1), as DR-6 proposed.
  5. The READY flag is harness-neutral: written per harness by its adapter (Claude Code via hooks; others via theirs).
- **Why:** operator: the point is that agents receive and act; also observed that typed lines arrive mid-task, sit
  unsent when Enter did not take, and that he deletes them; a false "delivered" must be impossible.
- **Rejected:** A (ring only when no client attached: an attended-but-idle agent never acts); B (today's always-ring:
  types into a busy session and into a person's half-written prompt).
- **Change requests to step 4 (architect, next revision):** CR-O2a define and measure "communication under way /
  last activity" per session: terminal input (typing), agent output (transcript growth, turn start/stop events), and
  peer traffic for that session (messages sent/received on its inbox and DM topics, hook surfacing events), so the
  300 s fallback and the conversation check have a measured signal; CR-O2b the 300 s fallback as a sidecar watchdog
  independent of the READY flag, with its own audit row; CR-O2c conversation detection (operator<->agent and
  agent<->agent) as a separate signal from the agent's busy flag.

### 2026-11-03 — O-3 who may be main before signed profiles: A with a tightening (operator ruling, after consultation)
- **Chose:** A with the tightening (operator: "A with the tightening"). Any interactive session of the project may
  become main (first claim wins; operator pin overrides), as built, PLUS: (1) a renewal or takeover counts only on a
  HUMAN-initiated prompt, never on automated activity (timers, background prompts, injected lines); (2) a session where
  the operator actually prompts can take main over faster than lapse + quiet + cooldown when the holder shows no human
  activity; (3) a loud "mail waiting, main not reading" signal to the operator.
- **Why:** consultation (docs/reports/T-3353-O3-consultation/: opencode A, antigravity A, codex C if operator attention
  is binding, else A defensible). All three: B is degenerate (all sessions share one agent id; no named agent = no main
  at all, the operator's objection). All three named the residual risk of A: a forgotten session renewing forever via
  automated prompts; the tightening is their shared fix.
- **Rejected:** B (filters nothing, adds a no-main outage); C (no pin = no main, mail waits; pin fatigue).
- **Change request CR-O3:** architect specifies (a) how "human-initiated prompt" is detected per harness (adapter;
  ties to CR-O2a's activity measurement), (b) the fast-takeover rule and its bound, (c) the waiting-mail signal and
  where it lands (session-start alerts, cockpit, operator).

### 2026-11-03 — O-4 switch off Claude Code SendMessage for this project: A (operator ruling)
- **Chose:** A (operator: "A"). `permissions.deny` for `SendMessage` in this project's settings (6.10.2, ends RA-5);
  the in-process sub-agent tool (Agent/Task) stays allowed and logged (6.10.1) while the framework's dispatch uses it.
- **Why:** one audited route for agent mail (FX-1), consistent with O-3; the replacement route (T-3386) is done.
  Scored A +16, B 0, C -9 (product set).
- **Rejected:** B log only (a second, unaudited mail channel stays open); C disable both (breaks framework dispatch).
- **Left open:** the deny is an operator act (6.10.2): delivered as a runme action under build item B-18; the session
  message prover (T-2876) uses SendMessage and must be re-based on TermLink before or with the deny.

### 2026-11-03 — O-5 the starting values of section 8: A, accepted as one package (operator ruling)
- **Chose:** A (operator: "A"). V-1..V-13 (and the related DR-3, DR-12, DR-14) accepted as starting values under FP-4:
  each is measured in operation, its measurement is an acceptance item of its build item, and it is tuned from the
  measurement. V-9's lease constants stand together with O-3's fast-takeover tightening (CR-O3).
- **Why:** starting values with built-in measurement correct themselves; 15 separate rulings would spend operator
  attention on details.
- **Rejected:** B one by one; C defer.
- **Left open:** V-14 (GP-4 offline, 8.2) and V-15 (security levels) are O-7's subject; V-16 (GP-11) stays open.

### 2026-11-03 — O-6 the four step-3 assumptions (operator rulings, all four recorded as below; not "as assumed")
- **OP-5 interim approvals:** NOT a new hub-admin verb. Approvals go through the existing surfaces: Watchtower and the
  operator's out-of-band channels (inventory T-3350, GP-11: ring20 named six channels, three two-way, Mattermost desk;
  GP-11 channel standard is AEF's). Operator: "we have Watchtower and we have out-of-band channels". CR-O6a: step 4
  replaces the hub-verb placement (10.4.2 / 6.x approval broker) with Watchtower + OOB channels.
- **OP-12 thresholds:** H-21 ("at most 10 approval events a day") is DROPPED as a quality threshold: it measures
  utilisation, not reliability (operator: "my car could be very reliable if I drive it once per year"). Thresholds must
  measure reliability: share of messages that reach the agent's conversation (HANDED_OVER) per attempt, every failure
  visible, zero false "delivered". Operator load may be tracked separately as a cost, never as quality. H-14 stays.
  CR-O6b: revise section 9 / FP-4 thresholds accordingly.
- **OP-22 who builds the receive side:** OD-7 as RULED 2026-10-06 applies: TermLink owns the adapter CONTRACT; each
  harness team owns its adapter (Claude Code included); OD-9 C for the receiver. The section 16 table ("TermLink owns the
  Claude Code adapter + hooks, OP-22 A in fact") is wrong on the rule and on the facts (AEF built the T-5014 hook gate).
  CR-O6c: correct section 16 to OD-7/OD-9; align the split with AEF explicitly.
- **OP-23 + standing rule:** keep every existing route working until its replacement is built, tested and proven in the
  field; never decommission implicitly; only with the operator's explicit, documented approval (operator: "never
  implicit ... explicit documented approval"). RA-9 (old rails unframed until PH-4) accepted under this rule. Recorded
  as a project standing rule in CLAUDE.md.

### 2026-11-03 — O-7 security levels: A, three fixed levels per project (operator ruling)
- **Chose:** A (operator: "A"). L0 base (today, every project), L1 separate accounts (agents never as root), L2 keyed
  (operator-signed policy records, confirmation for high-impact classes, encrypted DMs). The floor is never switched off
  at any level; raising a level is an approval, lowering is high-impact (XR-6); the Tier 0 override stays as logged
  break-glass (8.4, DR-13 A).
- **Why:** small and auditable. Scored A +14, B -3 (product set).
- **Rejected:** B a free per-item matrix (hard to oversee, invites weak combinations).
- **Step 4 status:** all operator questions O-1..O-7 ruled 2026-11-03. Change requests for the architect's revision:
  CR-O2a..c, CR-O3, CR-O6a..c, and O-1's follow-up (T-3663). GP-11 (V-16) stays open, owned by AEF.

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-10-06T10:13:26Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3353-arc-011-role-chain-step-4-architect.md
- **Context:** Initial task creation

### 2026-11-02T21:47:33Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-11-03T07:35:09Z — status-update [task-update-agent]
- **Change:** tags: +security
