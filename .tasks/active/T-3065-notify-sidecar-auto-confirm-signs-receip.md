---
id: T-3065
name: "notify-sidecar auto-confirm signs receipts with the per-agent key, not the
  dm-party fp — --await-ack senders never see them"
description: >
  scripts/notify-sidecar.sh:138 exports TERMLINK_AGENT_ID=<agent_id>, so every auto-confirm
  receipt (T-3053) is signed by the per-agent key: on this host claude-termlink resolves
  to 6738c073bbcc587a (termlink agent identity --resolve), while the dm topics it
  acks for are keyed on the shared fp d1993c2c3ec44c94 (.context/cron/notify-sidecar-agents.conf).
  A sender using channel post --await-ack derives the recipient from the dm topic
  name (crates/termlink-cli/src/commands/channel.rs:1240 derive_dm_recipient) and
  polls channel.receipts for THAT sender_id, so a receipt from 6738c073bbcc587a satisfies
  nobody: every --await-ack against this host exhausts and dead-letters even though
  auto-confirm is running. Measured live on dm:8e6fd77ec6f74b37:d1993c2c3ec44c94:
  receipts show 6738c073bbcc587a up_to=5 (sidecar) next to d1993c2c3ec44c94 up_to=3
  (manual ack this session). T-3053 proved a receipt APPEARS; it did not prove the
  receipt is the one --await-ack looks for — the same shipped-not-live shape as T-2876.
  Deliverable: sign the auto-confirm receipt as the dm party (--sender-id or identity
  matching --self-fp), or make derive_dm_recipient accept a declared alias; add a
  fixture asserting receipt.sender_id == self_fp; then re-run scripts/notify-rail-e2e.sh
  RECEIPT stage. Origin: T-3062 AEF sidecar alignment.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [bug, arc:reliable-comms, sidecar]
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
created: 2026-09-22T09:03:18Z
last_update: 2026-09-28T21:52:58Z
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
  - ts: '2026-09-22T14:57:18Z'
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
cost_estimate_proposed:
  - ts: '2026-09-22T14:57:41Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius:
      tier: 2
      effort: 8
    rationale: blast_radius=? (no-components-UNMEASURED-not-zero); tier=2 
      (workflow:build); effort=8 (lines=204,acs=4)
    rubric_sha: e4a00f38e801
---

# T-3065: notify-sidecar auto-confirm signs receipts with the per-agent key, not the dm-party fp — --await-ack senders never see them

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] AC1 — The auto-confirm receipt is signed by the key whose fingerprint EQUALS the
      `--self-fp` the sidecar is acking for, so `derive_dm_recipient` + `channel.receipts`
      find it. Relabelling via `--sender-id` alone cannot work and must not be attempted:
      hub `channel.rs:777` (T-1427) rejects a claimed sender_id that does not match the
      fingerprint derived from the signing pubkey. The signing KEY has to change, not the
      label — and that the hub refuses the shortcut is correct, not an obstacle.
- [x] AC2 — The mechanism is the documented precedence
      (`TERMLINK_IDENTITY_FILE > TERMLINK_AGENT_ID > TERMLINK_IDENTITY_DIR > host default`),
      scoped to the receipt post ONLY. The sidecar must keep exporting
      `TERMLINK_AGENT_ID` for everything else — that export is T-2292 per-agent identity
      and is not a bug.
- [x] AC3 — FAIL LOUD when no local key matches `--self-fp`. Today the receipt posts
      successfully and satisfies nobody, which is indistinguishable from working. A
      receipt that cannot be found by the sender is worse than no receipt, because the
      offset guard then records the topic as acked. Refuse and say so on stderr instead.
- [x] AC4 — The offset guard must not suppress the corrective re-ack. Guards currently
      record offset 49 on `inbox:cacc73ea32b121dd/010-termlink` from receipts posted under
      the WRONG identity, so after the fix the sidecar would skip re-acking and every
      already-"acked" topic stays unconfirmed forever. Invalidate the affected guards as
      part of this, or key them on identity so a signing-identity change re-arms them.
- [x] AC5 — Fixture asserting `receipt.sender_id == self_fp` (the task's own stated
      deliverable), `# guard-layer: source`, run against the real script via the
      `TERMLINK_BIN` stub seam, proven load-bearing by a mutant that restores the
      per-agent signing and must redden it.
- [x] AC6 — Re-run `scripts/notify-rail-e2e.sh --stages receipt` and record the result.
      Note its scope limit (T-3203 F1): it drives a `dm:` topic only, so it is a
      regression check on the dm path and NOT evidence about the inbox path.
- [x] AC7 — Live evidence on the real topic: after the fix, a receipt whose `sender_id`
      is the dm-party fp appears on `inbox:cacc73ea32b121dd/010-termlink`. Measured by
      read-back, not inferred from the post's exit code (T-2876).

### Human
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

## Findings

**F1 — fixed and proven live on the real topic.** Receipts on
`inbox:cacc73ea32b121dd/010-termlink`, grouped by sender, after the redeploy:

    3bba15e681b3a078  n=1  up_to=49   framework-agent-systemd (was always correct)
    6738c073bbcc587a  n=1  up_to=49   the OLD wrong-key receipt, harmless history
    d1993c2c3ec44c94  n=1  up_to=49   <- NEW: the dm party. This is the fix.

Exactly one receipt from the corrected identity, not a storm — the identity-keyed
guard re-armed the topic once and then held.

**F2 — the downstream effect is the one that mattered.** `channel unread --sender
d1993c2c3ec44c94` on that topic went **50 -> 0**, and all three agents' flags now read
`pending=0` (claude-termlink was 95). Before this, unread could never reach 0, so
`write_cycle` re-stamped `last_mail_ts` every cycle and the arrival flag was permanently
raised — a signal that is always on carries no information. It carries information again.

**F3 — T-3206's precondition is now satisfied.** That task closed as "blocked on T-3065",
and the conf note it left says: fix T-3065, confirm pending can reach 0, THEN add the
consumer line. Both conditions now hold. Adding a wake consumer for claude-termlink is
newly possible — deliberately NOT done here, because it is a separate change with its own
verification and this task's scope was the signing identity.

**F4 — claude-termlink-alt has not exercised the refusal path yet, and that is expected.**
Its declared self-fp is `6738c073bbcc587a`, which is the same fingerprint claude-termlink's
per-agent key resolves to — so the old wrong-key receipt happens to satisfy alt's cursor,
alt has nothing unread, and auto-confirm never runs for it. The refusal will fire the
first time alt has genuine unread mail. Its misconfiguration (declared fp matches no key
it can sign as) is real and still stands; it is now loud-on-use rather than silently
mis-signing. Fixture case 3 covers the path directly.

**F5 — answering "do we need to resend?".** The correction message to AEF does NOT need
resending: it was a plain `channel post`, verified byte-identical by read-back at inbox
offset 10 / sidecar offset 22, and is readable by them regardless of receipts. What
needed re-sending was the RECEIPTS, and the identity-keyed guard did that automatically
on redeploy rather than requiring manual guard deletion.

## Verification

bash tests/notify-sidecar-receipt-identity-fixtures.sh
# No regression in the T-3203 inbox enumeration that shares this file.
bash tests/notify-sidecar-inbox-fixtures.sh
# The script runs detached under nohup from cron, where a parse error is invisible
# until the rail is silently dark again.
bash -n scripts/notify-sidecar.sh
# AC1 — the fix signs by changing the KEY, never by relabelling. The hub refuses a
# mismatched --sender-id (channel.rs:777, T-1427), so a --sender-id here would be a
# receipt that fails to post rather than one that fails to be found.
test -z "$(grep -nE '^[^#]*--sender-id' scripts/notify-sidecar.sh)"
# AC2 — the per-agent export is PRESERVED; this fix is scoped to the receipt post only.
# Pattern deliberately stops before the "$agent_id" literal: the executing shell expands
# it to empty, so the fuller pattern fails for a quoting reason unrelated to the code.
# A gate that blocks for the wrong reason is how operators learn to --force past it.
grep -q 'export TERMLINK_AGENT_ID=' scripts/notify-sidecar.sh
# AC3 — the refusal path exists and is loud.
grep -q 'REFUSING to auto-confirm' scripts/notify-sidecar.sh
# AC4 — the offset guard embeds the signing identity so a signing change re-arms it.
grep -q '_receipt_identity" | tr -c' scripts/notify-sidecar.sh

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

**Symptom:** Peers using `channel post --await-ack` against this host never saw a
confirmation and their messages exhausted and dead-lettered — while our auto-confirm was
running, posting receipts, and reporting success on every local surface.

**Root cause:** `notify-sidecar.sh` exports `TERMLINK_AGENT_ID` (T-2292 per-agent
identity), so the auto-confirm receipt was signed by the per-agent key. But a sender
derives the recipient from the dm topic NAME (`derive_dm_recipient`) and polls
`channel.receipts` for that sender_id. On this host claude-termlink's declared self-fp
`d1993c2c3ec44c94` and its per-agent resolution `6738c073bbcc587a` are different keys, so
the receipt was correct, valid, verifiable — and filed under a name nobody was looking up.

**Why structurally allowed:** the two identity systems were introduced for different
reasons and nothing asserted they agree. The declared self-fp lives in a cron conf; the
signing identity comes from an env-var precedence chain; no check compared them. T-3053
proved a receipt APPEARS and that was accepted as proof the mechanism worked — the
identical shipped-not-live shape as T-2876, where a send returning success was taken for
delivery. Measuring the wrong end of the transaction is the recurring error: the sender's
exit code, the receipt's existence, the process's liveness. None of them is the property
that matters.

**Prevention:** (a) the sidecar now PROBES which local identity carries the declared
self-fp instead of assuming, and REFUSES loudly when none does — turning a silent
mis-signing into a named misconfiguration, which immediately surfaced a second broken
agent (claude-termlink-alt) nobody had noticed; (b) `tests/notify-sidecar-receipt-identity-fixtures.sh`
asserts WHICH identity the post runs under, with a mutant restoring the old behaviour
proving it load-bearing; (c) the offset guard is keyed by signing identity, so a future
identity change re-arms the ack rather than being suppressed by a stale guard.

The deeper generalisable rule is PL-392's sibling and is recorded in F2: a confirmation is
only real if the party waiting for it can find it. Assert on the waiting party's view,
never on the emitter's success.

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

### 2026-09-22T09:03:18Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3065-notify-sidecar-auto-confirm-signs-receip.md
- **Context:** Initial task creation

### 2026-09-28T21:52:58Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
