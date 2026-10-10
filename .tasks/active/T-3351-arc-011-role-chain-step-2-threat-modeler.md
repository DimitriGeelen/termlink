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
last_update: 2026-10-08T09:49:10Z
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

### 2026-10-08 — OQ-6 what "the operator" means as a sender until GP-11 (operator ruling)
- **Operator question answered first:** does this cut the operator off (runme, approvals)? No: terminal input (incl.
  voice), runme.sh, Watchtower ticks, inception decisions and Tier 0 approvals are local actions, not rail messages,
  and are untouched; peer mail keeps flowing (urgent from outside the allow-list is downgraded, never dropped). Only a
  rail message CLAIMING to be from the operator loses automatic top authority (mid-turn interrupt) until it can be
  verified.
- **Chose:** D — A for the security floor now: CR-10 accepted; until GP-11 is ruled the default allow-list is "own
  project" only and no message is operator-class unless it verifies against an operator key kept off agent hosts
  (PR-1b, SI-8). GP-11 must assess the already-authenticated channels (operator terminal, Watchtower login, the
  out-of-band channels) as operator channels (C folded into GP-11).
- **Why:** closes operator impersonation (TH-5; ADV-14, prompt injection) at no practical cost today. Scored D +37,
  A +23, C +21, B −27.
- **Rejected:** B (top of the authority model forgeable), C now (GP-11's design work), A without the C assessment.
- **Settles:** CR-10 accepted; TH-5 countermeasure SI-8/PR-1b.

### 2026-10-08 — OQ-7 held: the operator proposed a principle (not a ruling)
- **Operator's idea (restated, not ruled):** read access follows the five identity levels: an agent reads its own
  mail, a project reads all its agents' mail, no project reads another's; the hub is a mailman that reads the address
  to route and forward but not the letter; the fallback never crosses into another project; exceptions only by an
  explicit approved act (Tier 0 named). Operator asked for reflection and suggested external review.
- **Orchestrator's formalisation:** P1 need-to-know by identity level; P2 hub reads envelope only; P3 end-to-end
  encryption to the agent key or a project key, the hub may keep an encrypted copy (recovers a lost store, OQ-5 F-5);
  P4 break-glass by an off-host operator recovery key (today's Tier 0 hook does not bind root; depends on GP-11).
  It would answer OQ-7, OQ-10 and OQ-11 together.
- **Action:** external review by Codex and GLM (lesson 5.18), `docs/reports/T-3351-principle-review/`; then the
  principle comes to the operator as ONE decision; OQ-7/10/11 follow from it.

### 2026-10-08 — Confidentiality principle B′, replacing OQ-7, OQ-10, OQ-11 (operator ruling)
- **Review:** Codex and GLM both "adopt with changes" (comparison.md): P1/P3 conflict, pin keys first, envelope
  metadata is a residual, fan-out per-message keys, bounded ciphertext retention, break-glass waits on GP-11.
- **Guiding rule (operator):** purpose and intent over form: the right party gets the right information to do the
  right thing, and the wrong party gets nothing that enables misuse.
- **Chose:** B′ (operator: "B plus the custodial rule plus purpose-intent-over-form … yes to all as suggested"):
  1. Private channels are private by default; public channels are public.
  2. The hub is a mailman: it reads the envelope (routing, ladder, stuck detection, canaries), not the content; the
     envelope's who-talks-to-whom-and-when stays visible (accepted residual).
  3. Content is encrypted per message (random content key) and wrapped to each authorised reader's key (fan-out);
     separate encryption and signing keys; no shared project private key for readers.
  4. A project reads mail addressed to the project and mail that fell back or dead-lettered to it; nothing crosses
     into another project.
  5. Custody rule: each agent's private mail is also wrapped to a project custody key held by the project's
     supervisor, never by sibling agents. The custodian releases an agent's mail to a successor only when the home hub
     has declared the agent DEAD (R-44/R-60) AND the successor has taken over that agent's role in the same project
     under the R-67 lease by the explicit hand-over of R-62. While the original is alive, nothing is released (no
     duplicate work, continuity of the chain). Every release is logged (what, to whom). A resumed agent (R-65, same
     identity) reads its own mail with its own key.
  6. The hub keeps an encrypted copy with BOUNDED retention, so a lost receiver store can be recovered (OQ-5 F-5).
  7. Break-glass: an off-host operator key to which message keys are wrapped in advance, scoped and logged unlock;
     NOT real until GP-11 (today's Tier 0 hook does not bind root).
  8. Build order for step 4 to size: pin hub card-signing keys out of band first; then agent-to-agent DM encryption;
     then project mail; break-glass after GP-11. Receiver-side "show my own mail" tooling for debugging.
- **Settles:** OQ-7 (token holders see envelopes only), OQ-10 (hub keeps an encrypted copy), OQ-11 (hub operator
  sees envelopes only); RR-6 (a)(b) narrowed to "envelope metadata visible"; CR-9 answered.
- **Residual named:** custody key and local keys reachable by host root until OQ-2's per-agent accounts plus a
  separate supervisor account exist (RR-2).
- **Rejected:** A (siblings read each other's mail; undoes OQ-2), C (every sender picks readers), D (keeps
  "everyone with a token reads everything").
- **Open:** OQ-13 (what a new participant may read), GP-11.

### 2026-10-09 — OQ-8 fleet admission (CAND-17, CR-8; operator ruling)
- **Dialogue:** recommendation D re-presented after the reboot. The operator raised the initial handshake: today's
  first-contact trust has no good closure (seen with Greenfield and Workflow Designer: flagged, not closed), and
  everything cannot rest on Tier 0 alone. The orchestrator first answered with three trust layers (D′); the operator
  corrected the reading: authentication and authorisation requests ("a hub requests this and this") should become a
  Tier 0 EVENT that then has the available approval routes, not a single fixed gate. Orchestrator mistake recorded:
  D′ treated Tier 0 as a gate only the operator opens and designed around it. (No record of the Greenfield/832 case
  was found in tasks or registers; the ruling rests on the principle.)
- **Chose:** D″ (operator: "I follow your recommendation"; "proceed as suggested"):
  1. D: roster, hub-signed cards (sequence + TTL), home-hub binding as R-60.a (2e) rules, project root key,
     continuity (signed key rotation, "moved to", restored-roster detection); CR-8 incl. a hub signing key separate
     from the TLS certificate (R-61 change).
  2. Every admission and authorisation request is a Tier 0 event carrying the request (who, which anchor, what it
     asks), its evidence and a digest; an approval is bound to that digest (SI-13).
  3. Approval routes chosen by risk class: (a) policy route for anchored low-risk cases (e.g. a new project on an
     admitted hub, a hub enrolling over SSH the operator already set up), auto-approved and recorded; (b) human routes
     (cockpit, Watchtower, terminal) for anything new; (c) exceptions always human (unsigned hub-key change,
     competing claim, re-pin, restored roster); (d) operator-key signature after GP-11. Not the rail until an
     operator key exists (OQ-6).
  4. Every decision, automatic or human, lands in one audit trail; RR-18 tripwire now: every re-pin and
     `tofu clear` recorded and shown in needs-attention.
- **Settles:** CR-8 accepted; new CR-18 (Tier 0 event + route registry) added; RR-4, RR-14 (a)/(b) accepted;
  RR-18 interim control authorised (build task T-3384).
- **Residual named:** until per-agent accounts (OQ-2) and the operator key (GP-11), routes live on hosts where agents
  run as root, so a Tier 0 event is detection plus discipline, not a lock (RR-2, RR-18).
- **Rejected:** A (leaves RR-18 silent), B (operator approval of every project id: friction), C (keeps HMAC, CAND-17
  unanswered), D′ (Tier 0 as one operator-only gate with workarounds).
- **Open:** which cases are low-risk for the policy route (operator decision at step 3/4); charter rewording
  T-2470; enrollment codes wait on GP-11.

### 2026-10-09 — OQ-9 the proposed numbers of section 19 (operator ruling)
- **Chose:** D (operator: "D"): PN-3 to PN-12, PN-15 and PN-17 become [R~] starting values, changeable with
  evidence; every hypothesis among them (H-3 PN-5, H-4 PN-6, H-5 PN-7, H-6 PN-10, H-7 PN-11, H-8 PN-12, H-12 PN-15,
  H-13 PN-17) carries its measurement as an acceptance item of the first build, which step 4 writes into its plan.
- **Assumption recorded:** PN-15 = 10 minutes, now also the expiry of a Tier 0 event on a human route (OQ-8 D″).
  The operator was asked whether 10 minutes fits and did not answer; overturnable at any time.
- **Rejected:** A (no obligation to measure), B (step 3 without anchors), C (leaves caps and rates open).
- **Open:** PN-14 (OQ-14), PN-16 (OQ-15); PN-1, PN-2, PN-13 were ruled in OQ-3.

### 2026-10-10 — OQ-12 who may add and remove participants (operator ruling)
- **Dialogue:** operator chose A but made it situational: predefined agents with predefined routes (pointed at the
  ring20-manager orchestration pickup; not found locally) should configure whether they may pull in others, may be
  invited, and how they respond to invitations; asked whether "can be invited" equals "responds to invitations"
  (answer: no, a gate versus a behaviour). Operator asked why removal should be a Tier 0 event; orchestrator
  conceded it need not be per removal (authority can be granted in advance in a profile). Operator generalised:
  configure which agents/agent types may join which, per scope (e.g. auto-join within a hub), "boundaries where
  certain freedom exists".
- **Chose:** A as the default when no rule says otherwise, plus a boundary model (operator: "Yes, that's the ruling,
  with your recommended default"):
  1. Join: an existing participant's signed event, countersigned by the newcomer; leave: self; epochs and fencing
     as PR-30.
  2. Agent profile settings: may invite; may be invited (a gate, refusals visible to the inviter); responds to
     invitations (accept automatically / ask = Tier 0 event / decline; unanswered invitations expire visibly);
     may remove (nobody by default / those it invited / anyone in conversations it started).
  3. Rules = who (agent, agent type, identity level) may do what (invite, be invited, auto-accept, remove) towards
     whom, where (conversation, project, hub, fleet). Rules nest fleet > hub > project > agent profile; a lower
     level may only narrow. Outside the boundaries: a Tier 0 event (OQ-8 routes), never a silent refusal; a
     recurring approval can be turned into a rule.
  4. Rules are signed policy records (SI-21); only the operator or an approved route changes them; an agent never
     edits its own boundaries. Every decision names the rule that allowed it. Small fixed vocabulary, no free-form
     expressions. Same engine as OQ-8's policy route.
  5. Default when nothing is configured: may invite yes; may be invited yes; respond = accept automatically from
     its own project, ask for anyone else; may remove = nobody. Removal outside a profile = Tier 0 event.
- **Settles:** PR-30 join/removal; CR-15 extended (profile fields, invitation expiry); new CR-19 (the boundary
  policy model; R-63 allow-list and R-64 grants become instances of it).
- **Rejected:** B (operator approves every join and removal), C (anyone adds or removes anyone), D (adder removes,
  unbounded); per-removal Tier 0 events (orchestrator's first proposal, withdrawn).
- **Open:** step 3 designs the rule format and defaults, together with OQ-8's low-risk classes; OQ-13 (what a
  newcomer may read); OQ-17 (who starts a hand-over).

### 2026-10-10 — OQ-13 what a new participant may read (operator ruling)
- **Dialogue:** recommendation C (forward only, inviter may grant a history range at invitation, recorded). The
  operator proposed a simpler form: the newcomer asks for the latest relevant messages and the inviter decides what
  is relevant, because there is an intent to invite the agent. Orchestrator agreed it is better than C (relevance,
  not a range).
- **Chose:** C′ (operator: "Yes, C′ is the ruling"):
  1. Forward only by default.
  2. The invitation carries its intent (one field in the join event, visible to all).
  3. The newcomer may ask for relevant context, or the inviter attaches it with the invitation.
  4. The inviter chooses the relevant turns; they are shared as the original messages (message keys re-wrapped to
     the newcomer, B′), keeping signatures and order; only what the inviter itself can read; if the inviter is gone,
     another participant may answer.
  5. Every share is recorded and visible to all participants.
  6. CR-19 rules may forbid sharing or make it automatic for configured cases.
- **Settles:** 8.6.d; CR-15 gains the intent field and the share event; CR-19 gains "may share history".
- **Residual named:** a compromised inviter may share too much; visible, not prevented (8.6.e, RR-1, RR-3).
- **Rejected:** A (context pasted by hand, signatures lost), B (whole history on join, silent), C (range-based grant).
- **Open:** cost of re-wrapping message keys for long histories (step 4).

### 2026-10-10 — OQ-14 protections for the operator approval device and key (operator ruling)
- **Dialogue:** recommendation D (PR-27's four protections, notice + 15 min delay by risk class). The operator
  rejected the delay mechanism: a high-impact approval must not continue automatically, because the operator may not
  be watching the channel (silence must not count as consent). The operator wants OTP-style out-of-band
  confirmation, at high priority, in a separate inception, in close cooperation with (not delegated to)
  ring20-manager, which manages the Cloudron estate: ring20-manager builds the facility, TermLink caters for it with
  a way to configure which OTP engine is used and how to connect to it, because different framework users have
  different OTP engines. "next" was not taken as a ruling; the operator then confirmed: "Yes, that's the ruling".
- **Chose:** D‴:
  1. The operator key needs a second factor; approvals are single-use, digest-bound and short-lived (PN-15 10 min);
     an offline recovery key can revoke the operator key.
  2. High-impact classes (in the CR-18 route registry) need an active second, out-of-band confirmation; never take
     effect on a timeout; unconfirmed within PN-15 they expire visibly. Moving a class out of high-impact is itself
     high-impact.
  3. Target: OTP-style confirmation through a pluggable confirmation-provider contract (ask with digest and human
     summary; verify once against that digest; configured per installation as a signed policy record; changing the
     provider is high-impact; fail closed when the provider is unreachable; prefer remote verification so secrets
     stay off agent hosts).
  4. Interim, where no provider is configured (shown as such): explicit double approval at the terminal (approve,
     then confirm after a summary of exactly what happens) plus a notice to the operator's channel.
  5. PN-14 (15-minute notice delay) retired. RR-16 accepted (device plus second factor compromised can still
     approve; much harder, not impossible).
- **Inception filed:** T-3385 (high priority; owned by 010; ring20-manager builds the OTP facility for the
  Cloudron estate to the contract; feeds GP-11).
- **Rejected:** A (fixed list, delay proceeds on silence), B (no second confirmation), C (leave to GP-11), D (delay
  proceeds on silence).
- **Open:** the OTP engine choice and contract (inception); GP-11; initial high-impact class list (step 3).

### 2026-10-10 — OQ-15 per-message digest beyond 14 days (operator ruling)
- **Chose:** A (operator: "a and next"): no per-message digest beyond the 14-day stage memory; RR-10 accepted;
  PR-34, CR-16 and PN-16 dropped. Revisit trigger: if the OQ-9 first-build measurements show re-sends older than
  14 days, the question comes back.
- **Rejected:** B (one-year per-message record as a declared retention exception; unmeasured volume, no incident).

### 2026-10-10 — OQ-17 who may start a hand-over to another copy (operator ruling)
- **Chose:** D (operator: "D and next"): a live holder's signed statement plus the new copy's acceptance, or the
  operator; if the holder is dead the hand-over is a Tier 0 event (CR-18), approved by a policy route when B′'s
  conditions hold (home hub declared the holder DEAD AND the successor holds the R-67 lease for the same role in the
  same project), otherwise a human route; CR-19 rules may tighten it per project.
- **Settles:** 8.6.b.3; CR-15 extended; first low-risk policy class "DEAD + same-role lease successor" for step 3.
- **Residual named:** a compromised home hub declaring a live copy DEAD (TH-54), partly covered by PR-15's
  supervisor check before a resume.
- **Rejected:** A (every crash waits for the operator), B (operator always), C (new copy alone; forbidden by R-62.a).
- **Still open after OQ-17:** OQ-16 (A+ proposed; waits on 055's telemetry answer), then the unlinked RRs and CRs.

### 2026-10-10 — How to take the 20 unlinked RRs and CRs (operator ruling)
- **Counted against register 22.18:** RR-1, RR-3, RR-7, RR-8, RR-9, RR-12, RR-13, RR-15, RR-17, RR-20; CR-1, CR-2,
  CR-3, CR-4, CR-6, CR-7, CR-11, CR-12, CR-13, CR-17 (OQ-16 with RR-19 and CR-5 held for 055).
- **Chose:** D (operator: "D"): three single decisions first (CR-1 outage exception to "hub record first"; RR-8
  second live copy and role "main", tied to 055's F10 consult; RR-13 founding verbs bypass the sidecar), then four
  bundles, any item pullable: D4 message integrity (CR-2, CR-3, CR-4, CR-17, RR-17, RR-15); D5 circuits and restore
  (CR-7, CR-12, RR-20); D6 peer content (CR-6, RR-1, RR-3, RR-7); D7 authority and limits (CR-11, CR-13, RR-9, RR-12).
- **Rejected:** A (20 rounds), B (bundles without the three singles first), C (acceptance by silence).

### 2026-10-10 — CR-1 outage exception to "the hub record first" (operator ruling)
- **Chose:** A (operator: "A"): CR-1 as written. While the hub is unreachable and an established conversation runs
  on its circuit (R-7.e (1)): stages go to the receiver's hash-chained local log first and are replayed and
  reconciled on return; replies travel on the circuit and are recorded locally; the sender's state is computed
  from circuit receipts and labelled "unreconciled" (UNKNOWN only when neither hub record nor receipts can be read);
  the owed list is computed from the local log, labelled unreconciled. On return the hub record is the source of
  truth again (R-2.e unchanged).
- **Touches when step 1 is reopened:** R-2.a, R-2.e, R-14.o, R-15.o, R-26.o, R-44.a, R-58.a.
- **Rejected:** B (circuits pause, reverses R-7.e (1)), C (contradictory requirements).
- **Open:** reconciliation details (step 3); RR-15 names what the log cannot catch.

### 2026-10-10 — RR-8 second live copy and the role "main" (operator ruling)
- **Dialogue:** recommendation D (deliberate extras declare "not eligible for main"; healthy incumbent keeps main;
  only an unexpected copy gives "authority unknown", raised as a Tier 0 event). The operator put effective
  information flow first (nothing misrouted or stuck) and added detection: whoever sees two mains informs; to avoid
  an authority problem the hub decides; fallback "the longest running". Then added forwarding: once main is
  established, any other copy that receives mail for the role forwards it to main and tells the sender to send to
  main. Orchestrator added guards (readiness on "longest running"; role-addressed mail only; loud; once; no loops).
- **Chose:** D′ with forwarding (operator: "Yes, record it"):
  1. A deliberate extra declares "not eligible for main" in its operator-signed profile (CR-13, CR-19).
  2. Any agent or sidecar that notices duplicate mains reports to the home hub (a report, not authority).
  3. The home hub resolves at once: operator pin, else the longest continuously READY holder, else stable id. Both
     copies are told ("main, generation N" / "instance, main is X"); the operator and cockpit get a notice; the
     operator may override (audited). The loser's replies inside its own conversations stay valid (R-67).
  4. Afterwards any non-main copy receiving ROLE-addressed mail forwards it once to the main the hub currently names
     (checked by generation), keeping the original signature, recorded in the hub record, and tells the sender
     "delivered via Y; main is Z (generation N); resolve via the home hub". Conversation mail bound to an exact copy
     is never forwarded (R-62). If the target is no longer main, the message returns to normal resolution, never
     bounces. CR-3 identity dedupes a double arrival.
  5. CR-20 amends R-67.a ("two live copies give authority unknown" replaced by automatic resolution plus notice).
- **Residual named:** RR-8 narrowed to "the hub may keep the wrong copy until the operator overrides" (accepted).
- **Rejected:** A (any duplicate freezes main), B (hidden incumbent keeps everything, the 999 incident), C (guess
  without notice), D (unexpected case waits for the operator).
- **Also authorised:** reply to 055's F10 consult on these terms (sent 2026-10-10).

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
