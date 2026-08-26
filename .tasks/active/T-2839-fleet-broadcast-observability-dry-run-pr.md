---
id: T-2839
name: "fleet broadcast observability: dry-run previews, recipient agreement, label accuracy"
description: >
  fleet broadcast observability: dry-run previews, recipient agreement, label accuracy

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: [scripts/chat-arc-broadcast.sh, scripts/chat-arc-multicast.sh, scripts/check-fleet-recipient-agreement.sh, scripts/check-hubs-parse-agreement.sh, scripts/check-installed-binary-drift.sh, scripts/check-receiver-ack-lag.sh, scripts/check-unpaired-capture.sh, scripts/check-vacuous-verification.sh, scripts/lib/hubs-toml-walk.sh, tests/gitignore-framework-scope-fixtures.sh]
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
created: 2026-08-26T15:18:35Z
last_update: 2026-08-26T17:49:39Z
date_finished: 2026-08-26T17:49:39Z
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
---

# T-2839: fleet broadcast observability: dry-run previews, recipient agreement, label accuracy

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] Both broadcast paths resolve recipients through ONE shared definition
      (`hub_addrs_from_toml`, scripts/lib/hubs-toml-walk.sh) implementing the
      operator ruling of 2026-08-26: the fleet is EVERY CONFIGURED HUB.
- [x] That shared parser accepts indented TOML `address` keys. It shipped
      anchored on `/^address/` and silently dropped tab- and space-indented
      hubs. A hub the SHARED parser drops is unreachable by every broadcast
      path, which is precisely what the ruling forbids.
- [x] A differential guard (scripts/check-hubs-parse-agreement.sh) runs the two
      independent hubs.toml parsers over 7 TOML shapes and fails both when they
      DISAGREE and when they agree on fewer than all 7 — agreement between two
      implementations is not correctness.
- [x] Both broadcast paths carry `--dry-run`, rendering the RESOLVED recipient
      set and posting nothing (chat-arc-broadcast.sh:53, chat-arc-multicast.sh:35).
- [x] The receiver side has a canary (scripts/check-receiver-ack-lag.sh) that
      separates caught-up / behind / NEVER-ACKED, keyed on `up_to == null`
      rather than on lag, with a `--self-test` proving a never-acked sender is
      not rescued by a small lag.
- [x] The T-2687 fail-open guard is present in vendored `lib/pickup.sh` and its
      fixtures pass 12/12. A re-vendor had deleted it; reinstating a REGISTERED
      divergence is the register's purpose, not a G-062 violation.
- [x] Every entry in `.vendor-divergence.yaml` was audited mechanically by its
      PROPERTY rather than by marker count — 3 landed-upstream, 2 filed-upstream.
- [x] Every `# guard-layer:` marker is executable exactly as declared. The
      marker IS the invocation, so a marker naming a flag its own script rejects
      is a broken guard that reports ERROR rather than a verdict.
- [x] tests/gitignore-framework-scope-fixtures.sh passes 27/27.

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
       `bin/fw reviewer T-XXX 2>&1 | grep -q "Overall:.*PASS"` added to ## Verification.
-->

- [ ] [REVIEW] The "every configured hub" ruling is implemented the way you meant it
  **Steps:**
  1. `cd /opt/termlink && bash scripts/chat-arc-broadcast.sh --payload "ruling check" --dry-run`
  2. Compare the RESOLVED recipient list against the hubs you expect a fleet
     broadcast to reach.
  **Expected:** every hub configured in hubs.toml appears in the resolved set, and
  none is filtered out. Nothing is posted.
  **If not:** name the hub that is missing (or that should not be there). The single
  shared definition is `hub_addrs_from_toml` in scripts/lib/hubs-toml-walk.sh; both
  broadcast paths call it and nothing else resolves recipients.

- [ ] [REVIEW] Restoring a deleted vendored guard was the right call under G-062
  **Steps:**
  1. Read the T-2687 entry in `.vendor-divergence.yaml` (status: filed-upstream).
  2. Read commit be0fcfa68.
  **Expected:** you agree that reinstating a REGISTERED divergence which a vendor
  event deleted, and which upstream has not carried, is the register's purpose
  rather than a local patch of vendored code.
  **If not:** say so and I will revert be0fcfa68 and carry the loss upstream instead
  of holding it locally.

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
# Why not `cmd | grep -q PAT` (L-387): P-011 runs each line under `set -eo
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

test -f scripts/lib/hubs-toml-walk.sh && grep -q 'hub_addrs_from_toml' scripts/lib/hubs-toml-walk.sh
grep -q 'hub_addrs_from_toml' scripts/chat-arc-broadcast.sh && grep -q 'hub_addrs_from_toml' scripts/chat-arc-multicast.sh
grep -q '\-\-dry-run' scripts/chat-arc-broadcast.sh && grep -q '\-\-dry-run' scripts/chat-arc-multicast.sh
grep -q 'pickup_dedup_hash: envelope not readable' .agentic-framework/lib/pickup.sh
bash scripts/check-hubs-parse-agreement.sh > /tmp/.t2839-hubs.out 2>&1 && grep -q 'AGREE — both parsers return all 7 fixture addresses' /tmp/.t2839-hubs.out
bash scripts/check-receiver-ack-lag.sh --self-test > /tmp/.t2839-ack.out 2>&1 && grep -q 'self-test: PASS' /tmp/.t2839-ack.out
bash tests/pickup-failopen-fixtures.sh > /tmp/.t2839-pf.out 2>&1 && grep -q 'pickup-failopen-fixtures: 12 passed, 0 failed' /tmp/.t2839-pf.out
bash tests/gitignore-framework-scope-fixtures.sh > /tmp/.t2839-gi.out 2>&1 && grep -q 'gitignore-framework-scope-fixtures: 27 passed, 0 failed' /tmp/.t2839-gi.out

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

**Symptom:** a fleet broadcast silently reached fewer hubs than were configured, and
nothing on the receiving side measured whether anybody was reading.

**Root cause:** two independent hubs.toml parsers existed with nothing comparing them,
and the one promoted to "the shared definition" was the WEAKER of the two — its awk
anchored `/^address/` and therefore dropped tab- and space-indented address keys, which
are valid TOML. Separately, the ack rail instrumented only the sender side
(check-unconfirmed-delivery-freshness), so "nobody is reading this topic" was observable
only by a human running the verb by hand.

**Why structurally allowed:** recipient resolution had no preview, so the resolved set
was never observable *before* a send — the only way to learn who a broadcast reached was
to broadcast. And duplicated parsing was framed as a deduplication opportunity rather
than as a correctness question: collapsing N implementations into one selects a winner,
and selecting it by "which is shared" instead of "which is CORRECT" nearly replaced the
tolerant parser with the defective one.

**Prevention:** check-hubs-parse-agreement.sh keeps the two parsers honest and fails both
on disagreement and on agreement-below-expectation; the two are deliberately NOT merged,
because merging removes the redundancy that made the defect visible. `--dry-run` makes the
resolved recipient set observable before a send. check-receiver-ack-lag.sh gives the
receiver side the canary the sender side has had since T-2295.

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

### 2026-08-26 — the fleet is every configured hub
- **What changed:** the operator ruled that the fleet is EVERY CONFIGURED HUB. Acting on
  that meant one shared definition, and extracting it exposed that the implementation
  about to become canonical was the defective one.
- **Plan impact:** deduplication stopped being the goal and agreement-checking replaced
  it. The two parsers stay separate on purpose — merging them would delete the redundancy
  that made the defect visible in the first place.
- **Triggered:** c8e39802a (awk indent fix), 3fa4b9d06 (shared definition),
  scripts/check-hubs-parse-agreement.sh.

### 2026-08-26 — a vendor event deleted a registered divergence
- **What changed:** I certified a re-vendor safe after checking ONE of four registered
  divergences. It had deleted the T-2687 fail-open guard. Without it an unreadable or
  empty envelope hashes to the constant sha256("||") =
  565d240f5343e625ae579a4d45a770f1f02c6368b5ed4d06da4fbe6f47c28866, which COLLIDES: once
  that digest is in the dedup ledger every later envelope that also fails extraction is
  silently dropped as a duplicate. A fail-open detection path degrading into silent data
  loss — this arc's defect class, sitting on the pickup rail.
- **Plan impact:** a vendor event must be checked against EVERY registered entry,
  mechanically, by PROPERTY rather than by marker count. Marker counts were reassuring in
  both directions: T-2469 went 1 -> 5 mentions and T-2304 went 36 -> 38, while T-2687 kept
  its function names and had lost every guard inside them.
- **Triggered:** be0fcfa68 (restore from ed60a64ea), d26a0caab (mechanical audit of all
  entries; 3 of 4 proved genuinely carried, two by implementations better than ours).

### 2026-08-26 — an assertion that outlived its premise
- **What changed:** gitignore-framework-scope-fixtures asserted that
  `.agentic-framework/tools/ollama-tool-loop.py` must be IGNORED. T-2819's corpus work
  had disproved that premise: `git check-ignore -v` returns nothing, and
  agents/termlink/termlink.sh:853 execs the file preferring the vendored copy, so
  ignoring it would ship a dangling exec target.
- **Plan impact:** every other stale artefact this session was a LABEL outliving its
  mechanism, and those fail quietly. A stale ASSERTION fails LOUDLY IN THE WRONG
  DIRECTION — this one pointed a red FAIL at a correct fix and instructed me to undo it.
  Trusting the test over the evidence, which is the disciplined instinct, would have
  re-broken the very exec path the fixture exists to protect.
- **Triggered:** 978686900; the suite went 26 passed / 1 failed -> 27 passed / 0 failed.

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

**Recommendation:** GO

**Rationale:** every Agent AC is verified by a command in the Verification block above,
and each of those commands was run before it was written down — the grep patterns are the
literal strings the runs emitted. What remains is two rulings only you can make: whether
the resolved recipient set matches the fleet you have in mind, and whether reinstating a
deleted vendored guard is the right reading of G-062. Neither is an evidence gap.

**Evidence:**
- The shared recipient definition is wired into both broadcast paths and nothing else
  resolves recipients; the indent defect it shipped with is fixed and guarded by a
  differential test over 7 TOML shapes.
- `--dry-run` on both paths makes the resolved set observable before a send, which is what
  turns the ruling from a claim into something you can check in one command.
- The receiver side now has the canary the sender side has had since T-2295, and its
  self-test proves never-acked is not collapsed into caught-up by a small lag.
- The pickup fail-open guard is restored and its fixtures are 12/12; the whole divergence
  register was then audited by property, which is how the loss was found to be the only
  real one of four.
- One fixture was asserting a premise a later discovery had disproved, and it failed in
  the direction that argues for undoing a correct fix. That is the finding I would most
  want a second opinion on.

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

### 2026-08-26T15:18:35Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2839-fleet-broadcast-observability-dry-run-pr.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-1a26525e
- **Timestamp:** 2026-08-26T17:49:41Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** yes
- **Findings:** none

- **Layer-1 escalations:** 1
  1. **external-publish** (high) — External publish or release
     - matched: `broadcast`

### 2026-08-26T17:49:39Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
