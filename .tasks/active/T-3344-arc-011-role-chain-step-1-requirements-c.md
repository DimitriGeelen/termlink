---
id: T-3344
name: "arc-011 role chain step 1: requirements collector (re-cast confirmed requirements
  + review results; operator ruling A)"
description: >
  Operator ruled A (2026-10-04): the confirmed requirements document becomes step
  1 of the arc-011 role chain. A separate requirements-collector session (fw termlink
  dispatch) writes docs/design/interactive-agent-communication-01-requirements.md
  in the chain's skeleton from the confirmed requirements and both review comparisons,
  turns OD-1..OD-18 into the interview question list, and hands back a role-handback/1
  record. The orchestrator then interviews the operator one OD at a time.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [arc:arc-011]
components: []
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
created: 2026-10-04T14:28:12Z
last_update: 2026-10-06T07:06:30Z
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
bvp_scores_proposed:
  - ts: '2026-10-04T14:28:28Z'
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
---

# T-3344: arc-011 role chain step 1: requirements collector (re-cast confirmed requirements + review results; operator ruling A)

## Context

Chain: `docs/design/interactive-agent-communication-role-chain.yaml`, step 1. How a step runs until the driver
exists: `docs/design/roles/README.md` section 3. Operator ruling A recorded in Decisions.

## Acceptance Criteria

### Agent
- [ ] `docs/design/interactive-agent-communication-01-requirements.md` exists in the chain skeleton (front matter, version history, Answers, Glossary, Requirements R-n each with type, source, verification and at least one acceptance criterion, Conflicts, Earlier requirements, Open questions, D-1..D-3 with text equivalents), written by a separate role session, not this orchestrator
- [ ] Every confirmed requirement R-1.1..R-14.4 is marked kept, changed (to which R-n) or dropped with the reason; every OD-1..OD-18 appears as a numbered interview question with its options
- [ ] A `role-handback/1` record exists next to the output and its output sha256 equals the file
- [ ] The interview with the operator has covered every open question, one at a time, and the answers are in the output

### Human
- [ ] [REVIEW] Independent review: step 1 output meets its role card's completion conditions and is fit as input for the next role
  **Steps:**
  1. The orchestrator runs the review (external consultation per profile P4.4 until the panel exists); a reviewer never runs review commands itself
  2. Read the stored review answers next to the output
  **Expected:** no unresolved finding against the card's completion conditions 6.1-6.5
  **If not:** the findings go back to the role session; it remediates and hands back again
- [ ] [REVIEW] Step 1 output approved
  **Sovereignty:** operator decision only
  **Steps:**
  1. Read `docs/design/interactive-agent-communication-01-requirements.md`
  2. Decide the residual open questions and change requests, then tick
  **Expected:** you sign the requirements off as the base for step 2 (threat model)
  **If not:** say what to change; the role session revises

### Human (template notes)
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.

     ── Prefix routing (T-1811, T-1878): default to [REVIEWER] if Expected is grep-able ──
     If your Expected clause is grep-able / file-exists / structural (a deterministic
     shell check), prefer [REVIEWER] — that AC should be an Agent AC with the reviewer
     command in `## Verification` instead of a Human AC here. Only keep [REVIEW] if
     verification genuinely needs human taste (tone, feel, layout rhythm).
     See CLAUDE.md §AC Classification Guidance for the conversion rule.

     [REVIEW] example (genuine human judgment):
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error

     [REVIEWER] example (static-scan-verifiable — convert to Agent AC + Verification):
       - [ ] [REVIEWER] Block message names both bypass mechanisms
         **Steps:**
         1. Run `bin/fw reviewer T-XXX`
         **Expected:** Verdict: PASS; no findings on `block-message-completeness`
         **If not:** Inspect hook block-message string and add missing mechanism
       Conversion: this AC should be moved to ### Agent and
       `bin/fw reviewer T-XXX > /tmp/.rev 2>&1 && grep -q "Overall:.*PASS" /tmp/.rev`
       added to ## Verification. NEVER `... 2>&1 | grep -q ...` — that is the shape the
       Pipefail/SIGPIPE section below forbids, and this line used to prescribe it.
-->

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).
#
# ── Pipefail/SIGPIPE: grepping a command's output (L-387, T-2090, T-2743, T-2738) ──
#
# THE DEFAULT — redirect to a file, then grep the file:
#     cmd > /tmp/.out 2>&1 && grep -q "PATTERN" /tmp/.out
#     curl -sf "$(bin/fw watchtower url)/page" -o /tmp/.out && grep -q "PAT" /tmp/.out
# Correct at any output size, and `&&` keeps the PRODUCING command's exit code in
# the verdict. Reach for this first; the alternative below is the special case.
#
# NEVER `cmd | grep -q PAT` (L-387) — why: P-011 runs each line under `set -eo
# pipefail`. When grep matches it exits and closes stdin while cmd is still
# writing, cmd takes SIGPIPE, the pipeline exits 141 — verification "fails" with
# the pattern present. Captured 4× (T-1716, T-1838, T-1862, T-1863).
#
# THE EXCEPTION — capture first, grep the capture:
#     out=$(cmd 2>&1); echo "$out" | grep -q "PATTERN"
# Valid ONLY while "$out" fits the 65536-byte pipe buffer, and it is on you to
# know that it does. Above that the form inverts and becomes the very failure
# L-387 describes: echo blocks on the full pipe, grep -q exits, echo takes
# SIGPIPE, rc=141 (T-2743 — measured on a 146,366-byte Watchtower page, 3/3 runs,
# deterministic not racy; rendered routes run 50-200KB, so anything that curls a
# page is over the line). It also discards cmd's exit code, so a 404 yields an
# empty capture that grep merely fails to match rather than a failed line.
# If you do use it: single pipe only, no intermediate tail/awk/sed stage between
# capture and grep (T-2090) — the middle stage is what `grep -q` slams its stdin
# on, and grep scans the whole captured string anyway, so the `tail -3` was
# cosmetic. `echo "$out" | grep -q PAT`, nothing between.
#
# ── Asserting an ABSENCE: prove the search could have succeeded (T-3144) ──
#
# `! grep -q "PATTERN" file` exits 0 when the pattern is absent. It ALSO exits 0
# when the file was renamed, deleted, or is empty — so the leg cannot distinguish
# "the bad thing is not there" from "I could not look", and the gate reports green
# over a check that never ran. Pair every absence assertion with something that
# fails if the search could not happen:
#
#     test -f path/to/file && ! grep -q "PATTERN" path/to/file    # existence first
#     grep -q "KNOWN_MARKER" f && ! grep -q "PATTERN" f           # positive companion
#     cmd > /tmp/.out 2>&1 && ! grep -q "PATTERN" /tmp/.out       # &&-joined producer
#
# Count-equals-zero is the same defect wearing a different hat, and it is the one
# that bites hardest over a COMMAND's output rather than a file:
#
#     [ "$(cargo clippy --workspace 2>&1 | grep -c "^error")" = "0" ]   # WRONG
#
# If cargo is missing, or dies before emitting diagnostics, there are no `^error`
# lines, the count is 0, and the leg passes — a build gate that goes green
# precisely when the build could not run. Measured in this corpus, not invented.
# Keep the producer's exit code in the verdict:
#
#     cargo clippy --workspace > /tmp/.out 2>&1 && ! grep -q "^error" /tmp/.out
#
# T-3144 censused 2853 task files: 71 absence assertions, 41 already correct, 30
# not. The convention mostly works — this note is here so the next one is written
# right, because a vacuous leg is invisible until the day the path moves.
#
# TEST RUNNERS need a guard either way (T-2738). `set -e` is suppressed inside the
# `if` condition the gate runs each line in, so in `cmd1; cmd2` only cmd2 is the
# verdict — and the pass marker you grep for survives a partial failure: a suite
# printing "3 failed, 9 passed" satisfies `grep -q "9 passed"`, and generalising
# to `grep -qE "[0-9]+ passed"` matches the same output. Keep the exit code:
#     python3 -m pytest <file> -q > /tmp/.out 2>&1 && grep -q passed /tmp/.out
# or add the guard the exit code used to supply:
#     out=$(python3 -m pytest <file> -q 2>&1); echo "$out" | grep -q passed && ! echo "$out" | grep -q failed
#     out=$(bats <file> 2>&1); echo "$out" | grep -q '^ok 1 ' && ! echo "$out" | grep -q '^not ok'
# The close gate refuses the unguarded form. Bypass: FW_ALLOW_UNJUDGED_TEST_RUN=1.
#
# REHEARSING A LINE BY HAND DOES NOT REHEARSE THE GATE (T-2743). Your interactive
# shell has no `set -eo pipefail`. A line has returned 0 by hand and 141 under
# P-011, from the same directory, the same second. To rehearse for real:
#     bash -c 'set -eo pipefail; <your verification line>'
#
# Enforcement-baseline hint (L-398, T-1886): if you edited `.claude/settings.json`
# (added/removed/reorganised hooks), add `bin/fw enforcement baseline` to your
# Verification block. Otherwise the canonical hash diverges and `fw doctor`
# reports a FAIL ("Enforcement baseline CHANGED") that accumulates silently.
# Origin: T-1849/T-1730/T-1731 each added a legitimate hook without refreshing
# the baseline — FAIL sat for multiple sessions until T-1886 cleaned up.

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

### 2026-10-06 — OD-17 CAND-14 obligations and the unanswered state (operator ruling)
- **Chose:** A — the sender-visible unanswered state with escalation is recorded as already covered (R-28/OD-8 reply or no-action; OD-3 STUCK "not answered within 1 h"; OD-14 escalation; CAND-10 awaiting-reply deadline). New P1 (extends R-28): an owed-answers list per agent, computed from the hub record (OD-5), shown at session start/resume and in needs-attention ("you owe N answers, oldest T"). Work requests' accepted/declined, progress deadline and completed/failed go into R-41's later slice, with the CAND-1 task proposal as the "accepted" step.
- **Source:** RV2 items 50 and 52 — ring20-manager's 8 unanswered requests; "Explicit acceptance establishes who owes an answer" (Codex). Why they stayed unanswered is not in the record.
- **Rejected:** B full contract on every message (ceremony on simple questions); C already covered (the debtor stays blind); D defer all to R-41.
- **Left open:** ACKNOWLEDGED_NO_ACTION closes an obligation; escalation to another role holder belongs to OD-18.

### 2026-10-06 — OD-17 CAND-13 clock skew and version skew (operator ruling)
- **Chose:** A — P1: every deadline is measured on a single clock (the observer's own monotonic clock, from its own send time to when it sees the result), never by comparing two hosts' clocks, so STUCK is immune to skew. P2 clocks: NTP required and checked on every host (preflight check); each telemetry event records which host stamped it plus a skew estimate from the hub round trip; cross-host delays flagged past a bound. P2 versions: software and protocol versions on the agent card; "old version" shown distinct from "deaf"; an incompatible protocol version refused loudly; patch-number differences do not matter.
- **Facts:** .107 NTP-synchronised; no skew check exists elsewhere; hubs report version and protocol version (T-3345): .107 0.12.220, .121/.122 0.12.221, all protocol 1; PROTOCOL_VERSION_TOO_OLD exists unwired (T-2700, owner human, not decided here).
- **Rejected:** B all P1 (same value, more cost); C clocks only (floors say "too old", not "cannot receive X"); D defer (single-clock shapes STUCK's definition).
- **Left open:** skew bound (step 4); T-2700; 055's "one version estate-wide" (not proposed).

### 2026-10-06 — OD-17 CAND-12 preventing flooding (operator ruling, refined in dialogue)
- **Chose:** A — source control. P1: re-sends follow the message's own ladder (corrected OD-3: normal each rung twice, urgent continuous) with random jitter; an early re-send is refused and the sender told why; duplicates never reset it. P1: a cumulative cap on open messages per sending agent, starting at 100 (derived: answer rate ~20/h × 1 h deadline × ~5 peers — a guess until measured), adaptive once telemetry exists (grow while answered on time, halve when STUCK rises); over the cap sends wait at the sender, never dropped, shown in needs-attention. P1 storm prevention: hop limit on forwarded messages; three message classes — (1) protocol receipts (RECEIVED/STORED/HANDED_OVER) always flow, are never answered by anything (no ack of an ack), do not count against the cap; (2) automatic content (auto-responders, notifications, status broadcasts, canary pings, escalations, digests) carries the automatic marker, gets receipts, never triggers an automatic content reply (RFC 3834), counts against the cap, hop-limited; an agent may read it and answer deliberately; (3) deliberate content incl. REPLIED, normal rules; rule: nothing automatic answers anything automatic, receipts answer nothing, only an agent's decision creates content; circuit breaker toward a failing peer; hand-overs coalesced into one bundle at the receiver. P2: expiry, cancellation, receiver-advertised window.
- **Operator's framing and corrections, part of the ruling:** the risk is flooding (re-sending too often, piling up); adhere to the ladder; a cumulative per-agent cap derived from what the system can handle; back-off against broadcast storms from networking (CSMA/CD, spanning tree); the automatic-message definition must not block stage receipts.
- **Rejected:** B receiver-side limits (symptom, no loop or lockstep cover); C all P2 (first release could flood itself); D hub governor only (protects the hub, not agents).
- **Left open:** jitter width, adaptive-cap parameters, hop-limit value (architect, step 4); telling AEF the per-priority ladders replace D-600 for agent mail.

### 2026-10-06 — OD-3 correction: two polling ladders by priority (operator, during CAND-12)
- **Correction:** normal messages (priority < 5) poll each rung twice — 1 min, 5 min, 15 min, 1 h, 4 h, 1 day, 3 days, 1 week, 1 month, 1 quarter, 1 year; urgent (priority >= 5) poll the continuous 43-rung ladder (15 s … 2 years). The OD-3 record had applied the continuous ladder to all messages. States and stuck deadlines unchanged. A message's ladder also governs its re-sends before STORED, replacing D-600 for agent mail (normal ladder = D-600 extended to years). AEF to be told; pickup offset 314 carries the superseded form.
- **Confirmation:** read back; operator answered "next" without corrections; recorded as confirmed, can be overturned.

### 2026-10-06 — OD-17 CAND-10 yield-and-wake (operator ruling)
- **Chose:** A, P1 (the step-1 document said P2; the operator asked for it 2026-04-26 T-243 and 2026-05-25 T-1800) — a send may carry "awaiting reply by <deadline>" and the sender yields (ends its turn); the reply is handed over into the sender's session linked to the conversation (OD-2/OD-6 delivery, OD-5 record); if the deadline passes first the sender is woken with STUCK (OD-3); an idle sender that cannot be woken shows WAITING FOR RECIPIENT; a short blocking wait (seconds) remains for quick exchanges (`termlink agent ask` exists, blocks up to 30 s by default).
- **Rejected:** B P2 (agents keep polling, the April "absolutely not working"); C blocking only (holds the turn, tool time limits, cannot wait for a busy peer); D defer.
- **Bias noted:** second consecutive P1-above-document recommendation; the first slice grows.
- **Left open:** the default deadline when none is given (OD-3's 1 h answer deadline the natural default); a reply after STUCK still wakes the sender, marked late (orchestrator proposal).

### 2026-10-06 — OD-17 CAND-8 explicit mail-hub declaration (operator ruling)
- **Chose:** A with 14e–14h — P1 (the step-1 document said P2; CAND-6's "right hub" needs it): each project declares its mail hub by canonical hub id plus address, in its own files (travels with the project, OD-11); clients verify the reached hub reports that id (`hub_id`, T-3345), refusing and flagging any mismatch, never silently using another hub; `TERMLINK_RUNTIME_DIR` is for local runtime files only; the declared hub is what CAND-6's "right hub" checks. 14e: a hub restart or cert rotation keeps the canonical id (OD-12), no action. 14f: a new hub has a new id; clients refuse loudly; recovery is a restore (id travels with state) or an operator-approved re-home (Tier 2, Tier 3 if recurring) applied by a runme action with mail rescue (T-3343 pattern). 14g: the same id seen in two places is flagged, never guessed (OD-11 applied to hubs). 14h: the declaration names the home hub's id and address, so an agent on any host reaches it over authenticated TLS; unreachable means UNKNOWN and waiting, never fallback to a local hub; a project move includes re-homing, and the old hub answers "moved to X"; circuits may be set up via another hub but the record goes to the home hub (OD-5).
- **Operator's questions, answered in the ruling:** hub crash, new hub, several instances, agent on another host.
- **Rejected:** B P2 (CAND-6 depends on it); C T-3340 guessing only (single-host, directory-based, the class that failed); D defer (whether, not where).
- **Left open:** declaration file and format (architect, step 4).

### 2026-10-06 — OD-17 CAND-6 reachability as a visible per-agent state (operator ruling)
- **Chose:** A with 16e — P1: four fields on the agent's home-hub card (OD-11): receiver up, right hub (binding: the hub receiving the reports owns the agent's inbox), adapter present, last surface time (= latest real HANDED_OVER or confirmation, never a heartbeat); readable by peers (subject to OD-15 consent) and by the operator, landing in needs-attention when a field goes bad; R-29's CLEAR verdict requires recent surfacing progress, not just a fresh heartbeat (corrects R-29.e). 16e: the fields are computed at the home hub from the existing telemetry back channel (R-35, OD-5); no new channel; a silent back channel shows UNKNOWN, never DEAD (OD-3).
- **Operator's check, confirmed:** the hub sets up the circuit; established conversations run directly sidecar to sidecar; step events and a copy of each turn go to the home hub on a back channel, never in the message's own path (OD-1 C, OD-5, R-35.e).
- **Rejected:** B operator-only (sender stays blind, operator becomes relay); C P2 (delivery without visibility repeats the failure); D canaries only (one pair once a day; the claude-termlink-alt sidecar fooled every existing signal).
- **Left open:** how long without surfacing is bad for an idle agent with no mail (architect, step 4).

### 2026-10-05 — OD-17 CAND-4 how urgent is marked (operator ruling)
- **Chose:** A — `priority` integer −9..9 in message metadata, default 0, clamped at the receiver (journal-mirror.sh:164); urgent at ≥5 by default (threshold confirmed), a receiver may change its own threshold (INJECTOR_URGENT_THRESHOLD, notify-injector.sh:98); the queue orders by priority then time (notify-sidecar-api.sh:142); urgency honoured only from allowed senders (OD-15), others downgraded, never dropped. Build follow-ups: a `--priority` option in the send tooling; ask AEF to carry the field.
- **Rejected:** B yes/no flag (loses existing ordering); C no default threshold (senders cannot predict); D defer (already built).
- **Left open:** revisit the threshold after the canary has run; attention budgets (CAND-12).

### 2026-10-05 — OD-17 CAND-3 how arc-011 proves it works (operator ruling, after external review)
- **Chose:** A revised with staging — tests chosen per FAILURE MODE at the lowest tier that can see it; four tiers: fixture (simulated time for long schedules), live hub, one real agent + scripted peer through the INSTALLED scheduler and hooks per supported harness, real agents for round trips (incl. the three-agent many-to-many of R-1.e, ACKNOWLEDGED_NO_ACTION, resume refused); dimensions across tiers: two hubs over a circuit, crash/reboot at the worst moments, the installed system (clean install/upgrade/boot from the release artifact, systemd and not); negative controls kill-checked (green before the break, red after), with the three real failures as standing controls (second hub in another runtime dir, signing key under the wrong name, scheduler/hook not installed) plus a blocked reply leg at tier 4; continuous proof (daily canary from the installed artifact, rotated across hosts/pairs, own staleness detected, escalation landing shown; drill re-run after cert rotation, re-vendor, host reboot); mutant-per-push only for invariants (dedupe, ordering, consent, never-fallback). Staging: each dimension's tests become required when the thing they test exists; all required before arc-011 closes.
- **Path:** operator found "two real agents for every requirement" too heavy, added the missing one-agent tier, then asked for external review (`docs/reports/T-3344-cand3-review/`: Codex 16 findings, GLM 13, comparison; no disagreement between them).
- **Score corrections (operator):** usability is value to the user, not our test effort. D3: A −1→+2, B +1→−1, C −2→0, D +1→−1; C F-AUTONOMY −1→0. Totals A +63, C +21, B +2, D −21. Test effort belongs on the cost axis, handled by staging.
- **Rejected:** B tiered draft (passes all three real failures, per both reviewers); C two agents per delivery requirement (heavy and blind to topology, harness, install); D profile rule only (later steps read the requirements).
- **Also flagged (Codex 6):** R-29.e treats "fresh heartbeat and no flag" as CLEAR — to be corrected in the step-1 write-up.
- **Left open:** the test estate and agent pairs (step 5).

### 2026-10-05 — OD-17 CAND-2 (+ CAND-19 merged) per-stage idempotence per message id (operator ruling, refined in dialogue)
- **Chose:** A refined — a P1 requirement: a stage memory per `client_msg_id` (the settlement record, at the hub per OD-5): one stored copy, one hand-over, one reply per id. A duplicate is checked against the stage: not yet STORED → it is the first real copy and is stored (retransmit always works); id known but content lost → the receiver asks for a resend; STORED not HANDED_OVER → not stored twice, hand-over continues, sender told "stored, awaiting hand-over"; HANDED_OVER no reply → never handed over twice, sender told when it was handed over; REPLIED → the existing reply is returned again. Chasing an unanswered message stays with the OD-3 deadlines and OD-14 escalation; a duplicate never resets it. Retention follows OD-16. CAND-19 merged here (not asked again).
- **Operator's correction, part of the ruling:** "accept at most once" must not block a retransmit after loss, an injection still pending, or chasing the answer; idempotence is per stage, not per message.
- **Facts:** hub dedupe is 5 minutes only (dedupe.rs:41, T-2049); no receiver-side dedupe exists today; AEF D-600 retries up to monthly and OD-3 keeps messages alive up to 2 years.
- **Rejected:** B P2 (duplicates arrive with P1 retries); C defer (whether is the question); D the hub's 5-minute window.
- **Left open:** a duplicate id with different content (step 2, threat model).

### 2026-10-05 — OD-17 CAND-1 peer content is untrusted (operator ruling)
- **Chose:** A — new P1 security-invariant requirement: peer text is delivered framed as untrusted data; a request from a peer becomes a task proposal, never direct execution (AEF D-695; extends our pickup rule G-020/T-469 to all agent mail). A reply to something the receiver itself asked for is still data but may be acted on within the receiver's own task.
- **Score correction (operator query):** usability was scored −1 for delegation friction; corrected to 0 because the core rule already requires a task before any action, own-request replies are exempt, and orchestrated delegation goes through claims. A total +30 → +35; ordering unchanged.
- **Rejected:** B P2 (risk live before the rule); C defer to step 2 (identified and cheap); D reject (contradicts OD-15).
- **Left open:** framing markers (adapter, OD-7); threat coverage (step 2).
- **OD-17 candidates already settled by earlier rulings (recorded, not re-asked):** CAND-5 by OD-14; CAND-7 by OD-7; CAND-9 by OD-10; CAND-11 by OD-15; CAND-15 by OD-11 and OD-3. CAND-16 becomes live because OD-1 chose circuits and is walked as its own question.

### 2026-10-05 — OD-16 telemetry retention window (operator ruling)
- **Chose:** C refined — journey events kept until 14 days after the message reaches a final state (REPLIED, ACKNOWLEDGED_NO_ACTION, DEAD), never cut while the message is open (the OD-3 ladder can keep a message waiting up to 2 years); digests kept 1 year as an explicit forever-class exception with owner and reason (IW-2 rule); IW-2 count/size ceilings apply, trimming the oldest FINAL events first, loudly, and open-message events only as a last resort with a needs-attention entry.
- **Numbers:** 14 days and 1 year are the agent's proposal, assumed accepted (operator ruled "C" without naming others); can be overturned.
- **Correction recorded:** the step-1 document called 14 days "an existing operator ruling"; T-3304 IW-2 shows it was the agent's recommendation, for topics, assumed accepted. Fixed in OD-16.d.1.
- **Rejected:** A flat 14 days and B flat 30 days (both cut the journey of a still-open message); D until digested (trusts one mechanism, rejected in IW-2).
- **Left open:** ceiling values (architect, step 4).

### 2026-10-05 — OD-15 interrupt consent, respawn and the startup chain (operator ruling, refined in dialogue)
- **Chose:** A refined — (1) mid-turn urgent delivery only from senders the receiver allows by verified key (default: own project + operator); others downgraded to normal, never dropped; (2) mail may start an agent only for role/project addressing with nothing live, under a Tier-2 operator approval (one-off, logged) or a Tier-3 standing grant (budget, restart limit, allowed senders); (3) an exact instance that died mid-conversation is RESUMED (same transcript/session id) only if the home hub says DEAD (never UNKNOWN), the conversation is open, the transcript is resumable, and a grant covers it; a resume counts against the restart limit and a second death on the same message flags it to the operator (sender sees STUCK); (4) an instance that ended cleanly or cannot be resumed: dead letter, sender sees DEAD, re-addressing to the role is the sender's choice; a fresh copy never answers conversation mail; (5) the agent proposes promoting a Tier-2 approval to a Tier-3 grant after 3 recurrences, the operator approves; (6) R-9 startup chain stays as confirmed, its "start the agent" step follows (2)-(4).
- **Operator's additions, part of the ruling:** pre-approved situations, recorded, with promotion proposed by the agent; the distinction between addressing something not running (may start) and an exact instance that is gone (may not, except resuming a conversation that died mid-way).
- **Rejected:** B no consent (impersonation); C approve every interrupt (fatigue); D never interrupt (reverses R-19/OD-2).
- **Left open:** grant fields and storage (architect, step 4); the IAC-72 narrowing of R-9 (separate question, not raised); how urgent is marked (CAND-4, OD-17).

### 2026-10-05 — OD-14 alarms, escalation and the last rung (operator ruling)
- **Chose:** B — a daily end-to-end message canary between two real agents, checking RECEIVED, STORED, HANDED_OVER (receiver transcript) and REPLIED against the OD-3 stuck deadlines; a canary failure is an escalation entry, not an alarm (urgent-only alarm rule stands); the last rung lands in the main agent's session-start "needs attention" list (role holder per OD-18) plus `/canaries`, the daily digest and the 055 cockpit when it exists; proven by a negative-control test (stop the injector, the entry appears by the next session).
- **Rejected:** A no canary (3 October, the stray hub and the claude-termlink-alt sidecar all failed with nothing firing); C hub-steward agent (adds the dependency it monitors; T-3333 stays parked); D defer (last rung lands nowhere).
- **Left open:** the channel by which the urgent alarm itself reaches the operator.

### 2026-10-05 — OD-13 section 14 items (operator ruling)
- **Chose:** A refined — R-40 merged into R-28 (satisfied by OD-2/6/8); R-42 now ("delivered" only with a recorded HANDED_OVER); R-41 later slice with expiry, execution-time authorisation, fencing token; R-43 dropped (duplicate of R-9).
- **Rejected:** B keep all (duplicate); C drop all (loses R-42); D R-40 only.

### 2026-10-05 — OD-12 address rulings and stable hub id (operator ruling)
- **Chose:** D — D-599 circuit form stands; hub canonical id minted once (today's fingerprint prefix becomes it, nothing renamed), fingerprint becomes the hub instance id; sidecar: alias ends one release after T-3342 (forward + warn until then); ask AEF for amended D-660.
- **Rejected:** A (leaves cert rotation renaming inboxes); B D-660; C both names.

### 2026-10-05 — OD-11 name-to-id directory (operator ruling, with the operator's location insight)
- **Chose:** A — home-hub project identity cards from observed authenticated registrations; pid minted once by the framework (names display only); hubs exchange cards, never messages; only the home hub says dead.
- **Operator's insight, part of the ruling:** identity is the pid carried in the project's files; path and host are observed attributes; rename/move updates the card; a move to another host changes the home hub and the old hub marks "moved to X"; a running copy is a second instance unless declared a fork and re-minted; a pid first seen in a new place while the old is alive is flagged to the operator, never guessed.
- **Rejected:** B no directory; C AEF keeps it (non-AEF users have none); D per project (cannot report liveness).

### 2026-10-05 — OD-10 address session level (operator ruling)
- **Chose:** C — new requests to project + role (home hub resolves); conversation mail to the bound exact copy, dead letter if ended, never redirected; runtime ids never reused.
- **Rejected:** A role only (conversation context lost on restart); B exact only (every restart breaks addresses); D ids without a rule.

### 2026-10-05 — OD-9 receive side (operator ruling)
- **Chose:** C sequenced — AEF's receiver is the one receiver now where AEF runs (TermLink scripts retired there); `termlink sidecar` built to the same contract; switch per host after the two-agent acceptance test. Never two receivers on one inbox.
- **Rejected:** A permanent AEF receiver (core promise depends on another project's governance); B scripts (unpackaged); D defer (re-vendor creates the collision anyway).

### 2026-10-05 — OD-8 stage names (operator ruling)
- **Chose:** B — HANDED_OVER (transcript evidence, hook or typed); ATTEMPTED for a typed line without evidence; terminal ACKNOWLEDGED_NO_ACTION beside REPLIED.
- **Rejected:** A INJECTED (inaccurate for hook delivery); C/D two meanings under different names.

### 2026-10-04 — OD-7 readiness (operator ruling)
- **Chose:** C — harness adapter contract (READY/BUSY/NOT RUNNING + evidence); Claude Code adapter = hooks; opencode adapter (055); parity test per release; screen classifier diagnostic only.
- **Rejected:** A screen as fallback (still gates typing); B hooks without a contract (readiness built twice); D screen as main signal (R-22).

### 2026-10-04 — OD-6 already-running sessions (operator ruling)
- **Chose:** B — no forced relaunch; mail delivered through the existing PostToolUse/Stop hooks at yield points; relaunch through the reachable launcher on natural restart; idle unreachable sessions show WAITING FOR RECIPIENT.
- **Rejected:** A/D forced relaunch (interrupts every agent, loses context); C listing only.

### 2026-10-04 — OD-4 and OD-5 (operator rulings)
- **OD-4 chose A:** two calls; RECEIVED informational, STORED releases the sender; may travel together on the hub path with two timestamps.
- **OD-5 chose record-first:** the hub record (R-12.1 telemetry) is the single truth; the callback is a fast notice (one or two attempts); a returning sender reads the record; states are computed from it.
- **Rejected:** OD-4 B one call (cannot name a receiver-side storage failure on a circuit); OD-5 callback-as-truth (fails exactly when the sender is down; two paths diverge, 055 M3).

### 2026-10-04 — OD-3 polling ladder and "stuck" (operator ruling)
- **Chose:** the operator's ladder, continuous (43 polls: 15 s .. 2 years, no gaps), plus five message states (waiting, waiting for recipient, stuck, unknown, dead) with stuck deadlines per step (accept 1 min, hand-over 2 ticks after ready, answer 1 h; urgent 15 s / next tool call / 5 min).
- **Why:** keeps the operator's standard; "stuck" = overdue while the responsible party is reachable, distinct from unknown and waiting-for-recipient, so alarms stay meaningful.
- **Rejected:** B short wait then stop (operator keeps polling); C AEF retry ladder only; D retention cap.

### 2026-10-04 — OD-2 urgent into a busy agent (operator ruling)
- **Chose:** B — urgent content via the harness hook channel (next tool call / end of turn); never typed into a busy prompt; idle agents get the normal inject. SQ-4 reconciled: urgent bypasses the wait, never the check.
- **Why:** meets the operator's immediacy rule with no typing risk; mid-turn hook delivery verified in this session.
- **Rejected:** A doorbell typed into a busy prompt; C harness interrupt (none exposed today); D wait only.

### 2026-10-04 — OD-1 cross-host send path (operator ruling)
- **Chose:** C — hubs set up a circuit; established conversations run sidecar-to-sidecar; the hub path is the fallback. Built after identity hygiene, the hub directory with liveness, and the hub-path conversation binding.
- **Why:** the only option that delivers the operator's conversation model; Codex, GLM and 055 accepted it with conditions in round 2 (set-up via hubs, per-circuit credentials, one delivery contract and log).
- **Rejected:** A hub carries cross-host (no circuit); B sidecar-to-sidecar for all mail; D binding only until measured (AEF's dissent; its measurement goes into the first build).
- Log: docs/design/interactive-agent-communication/interview-step-01.md

### 2026-10-04 — arc-011 requirements become step 1 of the role chain (operator ruling)
- **Chose:** A — re-cast the confirmed requirements document plus both review comparisons as step 1; OD-1..OD-18 become the interview questions, asked one at a time.
- **Why:** keeps everything the operator confirmed; the open decisions become the interview; the chain then adds the threat model every reviewer found missing.
- **Rejected:** B fresh interview (a 16th restatement); C skip the chain for arc-011; D defer until the re-vendor driver.

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

### 2026-10-04T14:28:12Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3344-arc-011-role-chain-step-1-requirements-c.md
- **Context:** Initial task creation

### 2026-10-04T14:28:28Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
