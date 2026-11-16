---
id: T-3366
name: "Hub canonical id minted once and persisted; fingerprint becomes instance id
  (OD-12)"
description: >
  Mint each hub's canonical id once and store it with its runtime state (existing
  hubs adopt today's fingerprint-derived id), so a certificate rotation no longer
  renames inboxes; end the sidecar: alias one release after the re-vendor.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [arc:arc-011]
arc_id: arc-011
components: []
related_tasks: [T-3344, T-3357, T-3726]
plan_unit: P-10
depends_on: []
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
created: 2026-10-06T10:16:54Z
last_update: '2026-11-16T05:45:23Z'
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
  - ts: '2026-10-06T10:17:44Z'
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
  - ts: '2026-10-18T22:32:57Z'
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
  - ts: '2026-11-16T05:45:23Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      AS_DELIVERY_TRUTH: 0
      AS_FAILURE_SURFACED: 0
      AS_OPERATOR_SIMPLICITY: 3
      D1: 0
      D2: 0
      D3: 0
      D4: 0
      F-TRUST: 0
      F-SURFACE: 0
      F-CONV-RECALL: 0
    rationale: 'AS_DELIVERY_TRUTH=0 (L0: no signal); AS_FAILURE_SURFACED=0 (L0: no
      signal); AS_OPERATOR_SIMPLICITY=3 (L3:keyword=rollback); D1=0 (no-signal); D2=0
      (no-signal); D3=0 (no-signal); D4=0 (no-signal); F-TRUST=0 (L0: no signal);
      F-SURFACE=0 (L0: no signal); F-CONV-RECALL=0 (L0: no signal)'
    rubric_sha: e4a00f38e801
    driver_set: product
---

# T-3366: Hub canonical id minted once and persisted; fingerprint becomes instance id (OD-12)

## Context

OD-12 ruling (log OD-12 point 3). T-3345 already exposes hub_id in hub.version. Minting location from step 4. Ruling summary: docs/reports/T-3344-step1-rulings-summary.md. Do not start before the step-8 plan confirms scope.

Arc: arc-011. Rulings summary: `docs/reports/T-3344-step1-rulings-summary.md`.

**Background / trigger:** The step-8 plan `docs/design/interactive-agent-communication-08-plan.md` v0.3 was approved
by the operator on 2026-11-12 (G-0, T-3357 `## Decisions`) and confirms this task's scope as plan unit **P-10** (build,
PH-2, T1 hardening): B-1, minted hub id + hub signing key + `hub.version` fields (DR-1 A, DR-2 A). Serves B-1, K-9a.
Re-scoped by PD-2: ending the `sidecar:` alias moves to the T4 decommission P-53 (T-3726, NID).

**Problem / hypothesis:** The hub's identity is its TLS fingerprint, which changes when the certificate is regenerated;
signed state (revocation list P-15, signed dead letters P-16, cards P-48) needs a stable id and a hub signing key first.

**Goal:** A canonical hub id minted once and persisted, a hub signing key, and the `hub.version` fields.

**Expected outcome:** The B-1 specs pass; existing hubs adopt the fingerprint-derived id; a re-mint beside existing
state is refused.

**Rollback:** the id file stays; clients ignore the new fields.

Depends on: gate G-0 (decision: plan approved 2026-11-12; satisfied)
Unblocks: P-11 (T-3363), P-15 (T-3703, security), P-16 (T-3704, security), P-31 (T-3713), P-48 (T-3722); part of gate G-2.

## Acceptance Criteria

### Agent
- [x] Canonical id persisted in the runtime dir and survives cert regeneration (unit e36_certificate_rotation…; L4 t3366_hub_identity rotation test; mutant m1 killed)
- [x] Existing hubs migrate to their current fingerprint-derived id (no inbox renamed) (runme-20261116T073957Z rc=0: local cacc73ea32b121dd, .122 22c19fedafd73da2, .121 1389a831016c4bf1, greenfield aef925c6d2f4ac54, each adopted before the restart and verified unchanged after; live hub.version serves hub_id + signing_key)
- [x] The B-1 specs pass, including the hub signing key and the `hub.version` fields (DR-1 A, DR-2 A) (docs/reports/T-3366-evidence.md; live `hub.version` serves `hub_id` + `signing_key`)
- [x] A re-mint beside existing state is refused (FCW-79 unit + L4 legacy test; mutant m2 killed)
- [x] The unit's fixture suite exits 0 with a PASS line per B-1 E-n and its mutant list is all killed (FP 3.1.b) (evidence table: E-36, E-37, E-126, E-166 PASS; E-38 is P-11's, E-163 not applicable, both stated; every mutant in docs/reports/T-3366-mutants.md killed, run on the deployed commit)

## Verification

env -u PROJECT_ROOT cargo test -p termlink-hub hub_identity > .context/working/.t3366-v1.out 2>&1 && grep -q 'test result: ok. 26 passed' .context/working/.t3366-v1.out
env -u PROJECT_ROOT cargo test -p termlink --test t3366_hub_identity > .context/working/.t3366-v2.out 2>&1 && grep -q 'test result: ok. 3 passed' .context/working/.t3366-v2.out
env -u PROJECT_ROOT cargo test -p termlink-sidecar --test t3366_hub_key_pin > .context/working/.t3366-v3.out 2>&1 && grep -q 'test result: ok. 9 passed' .context/working/.t3366-v3.out
grep -q 'PASS: all 26 mutants turned their test red' docs/reports/T-3366-mutants.md

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

### 2026-11-16 — build start: deployment order is part of the unit
- **What changed:** every runme run restarts the local hub onto the installed binary (step 5, `restart_hub`) and step 6 upgrades the fleet hubs. AL-17 FCW-79 makes a hub with prior state and no `hub.id` refuse to start. Shipping the binary without the migration in front of the restart would take every hub down on the next run.
- **Plan impact:** the legacy migration (AL-17 "one runme action per hub") is built here, not left to P-31: a CLI verb `termlink hub adopt-id` (writes `first16(current fingerprint)`, or random with "no certificate", only when `hub.id` is absent; idempotent; refuses a differing existing id), and runme steps 5 and 6 run it with the NEW binary before each restart. P-31 (T-3713) keeps the .107 encryption keys and key cards.
- **Slices:** S1 hub side: `hub.id` + `hub.signing.key` (FCW-79..82), `hub.version` reads the file and adds `signing_key`, `adopt-id` verb, E-36/E-37-hub/E-166-hub fixtures + mutants. S2 runme steps 5/6 migration-before-restart + fixtures. S3 sidecar signing-key pin (FCW-83/84, E-37 sidecar half). S4 `sidecar:` topic forward-with-warning (AC 3). E-38 is P-11's (T-3363) and is not built here.
- **Triggered:** none yet.

### 2026-11-16 — S1 + S2 built: spec reading on "prior state"
- **What changed:** AL-17 step 1 lists `hub.cert.pem` as prior state, but E-36.2/E-36.4 start a "cert only (legacy hub)" directory and expect it to keep `first16(fingerprint)`, which only AL-17's first-start branch gives. Built to E-36 (the tested reading): prior state = `hub.secret`, `hub.signing.key` or any topic log under `bus/` (what makes an id visible to peers); a certificate alone mints from the certificate. A real legacy hub always has a secret, so it still meets FCW-79 and goes through `adopt-id`. **Superseded the same day:** review r1 #3a objected that the agent settled the contradiction alone, toward the less fail-closed side; the operator ruled B (see Decisions): the certificate IS prior state, as AL-17 step 1 says.
- **What changed (2):** the hub mints before it writes its own secret/certificate in `prepare_start`, so a brand-new TCP hub gets a random id (CR-6.42), not one derived from the certificate it is about to create.
- **S3 design (AL-18 sidecar half):** one module `hub_key_pin` in termlink-sidecar, called from the existing `check_pin` in send.rs and pull.rs after the hub-id match, with the `hub.version` reply already in hand. Pin stored per hub id in store state `hub-key-pin-<hub_id>`. No `signing_key` offered (pre-T-3366 hub): unchanged behaviour, status says "not offered". No pin yet: pin only when the hub address is the local socket or TLS under the TOFU-pinned certificate, write the pin, needs-attention "hub signing key pinned" (FCW-84 otherwise: no pin, hub-signed data unverified). Pin differs: needs-attention naming both fingerprints, coalesced per day; `verified_hub_key()` returns None, so every hub-signed object P-15/P-16 will check is refused (FCW-83). Mail posting and pulling are NOT held on a key mismatch: there are no hub-signed objects yet, and holding mail would switch off a working route (NID rule); the spec refuses hub-signed objects, not mail. Status gains `hub_signing_key {pinned, offered, state}` (OBS-36). Pin changes only by a runme re-pin action (later, with P-15).
- **Plan impact:** S2 covers both local restart points (step 5 `restart_hub`, step 5a `install_hub_unit`) and the fleet (step 6), where the runtime dir is read from the running hub's own environment; an unconfirmable dir or an adopted id that differs from the served id fails the hub without swapping.
- **Triggered:** none.

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

### 2026-11-12 — plan G-0
- **Chose:** re-scope as proposed in plan 08 v0.3 section 7.1 (approved by the operator 2026-11-12, ruling A with PD-2, recorded in T-3357): this task is unit P-10 (minting, B-1); ending the `sidecar:` alias moves to the T4 decommission P-53 (T-3726), which needs the operator's own approval (NID, G-8).
- **Why:** minting is T1 hardening that signed state depends on; ending an alias switches a working route off, so it cannot ride along in a build task.
- **Rejected:** keeping the alias end here (would decommission implicitly, against the NID rule).

### 2026-11-16 — when to build the auth chain (operator ruling)
- **Chose:** A, "A, start T-3366 after the hand-over" (operator, 2026-11-16): build in this session, right after the T-3807 runme hand-over.
- **Why:** unsigned mail stopped a week ago (measured: 204/204 inbox messages of the last 7 days signed and valid), but a valid signature binds only the key, not the project; T-3366 → T-3704 is the chain that binds them.
- **Rejected:** B next session; C T-3704 first on a temporary key (breaks the security order); D park.

### 2026-11-16 — is a certificate-only directory "existing state"? (operator ruling, review r1 #3a)
- **Chose:** B, "as recommended please" (operator, 2026-11-16): the strict AL-17 step 1 reading. A certificate counts as prior state; a certificate-only directory refuses to start (FCW-79) until `hub adopt-id` gives it the certificate-derived id E-36 expects.
- **Why:** the spec contradicts itself (step 1 lists the certificate; E-36.2/36.4 start a cert-only directory). B fails closed; its only cost is an `adopt-id` step runme already runs. Real legacy hubs always have a secret, and a new hub mints before its certificate exists, so neither is affected.
- **Rejected:** A (keep the E-36 reading: a directory that lost every other file would mint silently, possibly from a rotated certificate); C (change request first, A meanwhile).

### 2026-11-16 — closing after three one-seat review rounds (operator ruling)
- **Chose:** A, "A, as recommended" (operator, 2026-11-16): one confirmation round on the final commit with BOTH seats once opencode's usage limit resets; close only on two PASS; a FAIL from either goes to the operator with the dispositions, not to another round.
- **Why:** the close ruling (T-3800, 2026-11-15) asks for a positive review from both seats; opencode never ran in rounds 1–3 (usage limit), and the 2–3 round cap was reached.
- **Rejected:** B close on evidence alone (departs from the ruling); C operator judges in Watchtower; D wait a day.

### 2026-11-16 — the `sidecar:` forward criterion (operator ruling)
- **Chose:** C, "C, split it into its own task" (operator, 2026-11-16). The criterion "Mail to sidecar: topics forwarded with a sender warning until the alias end release" moves verbatim in intent to **T-3809** (build, arc-011, horizon next), where it is built as a hub forward envelope carrying the original signed message.
- **Why:** one task, one deliverable: alias handling is not id minting, and PD-2 already moved the alias end to its own unit (P-53, T-3726). A plain copy would fail the receiver's signature check (the signature covers the topic), so the forward needs its own design. No `sidecar:` post in 42 days, so the split carries no delivery risk.
- **Rejected:** A build it here (keeps T-3366 and T-3704 waiting on a route idle for six weeks); B warn only (departs from R-71.a); D defer inside T-3366 (same wait as A).

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

### 2026-10-06T10:16:54Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3366-hub-canonical-id-minted-once-and-persist.md
- **Context:** Initial task creation

### 2026-11-16T05:45:22Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)
