---
id: T-3203
name: "notify-sidecar probes dm: only, so the arrival flag never advances for inbox: mail"
description: >
  probe_mail enumerates --prefix dm: only. It writes last_mail_ts, which the injector keys on, so self-addressed inbox: mail never wakes the rail. T-3201 fixed the mirror; this is the remaining blocker.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
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
created: 2026-09-28T19:15:06Z
last_update: 2026-09-28T19:15:06Z
date_finished: null
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

# T-3203: notify-sidecar probes dm: only, so the arrival flag never advances for inbox: mail

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] AC1 — `probe_mail` enumerates self-addressed `inbox:` topics ALONGSIDE `dm:`, so
      `last_mail_ts` advances on inbox mail. The two selectors differ and must stay
      distinct: `dm:` filters on the identity FINGERPRINT (`dm:<fp_a>:<fp_b>`),
      `inbox:` filters on the PROJECT name (`inbox:<circuit>/<project>`). Reusing the
      fp predicate on inbox topics matches nothing and would look like a working fix.
- [x] AC2 — Self-identity is a declared constant with an env override
      (`FW_SIDECAR_SELF_PROJECT`, default `010-termlink`), never `basename $PROJECT_ROOT`
      — T-2815/T-2816: a path-derived slug is wrong in a worktree. Same constant the
      T-3201 mirror fix uses, so the two halves of the rail cannot disagree about who we are.
- [x] AC3 — Another project's mailbox is NOT probed. 22 inbox topics exist here and 95 of
      232 records belong to AEF's ephemeral e2e identities; probing those would auto-confirm
      and journal another project's mail, then inject it into our prompt.
- [x] AC4 — `dm:` behaviour is unchanged (the change is purely additive), and the
      `TERMLINK_NOTIFY_TEST_TOPICS` test seam still short-circuits enumeration entirely.
- [x] AC5 — Fixture suite `tests/notify-sidecar-inbox-fixtures.sh` carries
      `# guard-layer: source`, runs against the REAL script via the `TERMLINK_BIN` stub
      seam (not a copy of the selector), and is proven load-bearing by a mutant: reverting
      the inbox arm must redden the inbox assertions and leave the dm: control green.
- [x] AC6 — Live premise verified BEFORE claiming the rail works: `channel unread --sender`
      returns a usable count on a self-addressed `inbox:` topic. If it does not, the fix is
      inert and this task must say so rather than report success.

- [x] AC7 — The auto-confirm side effect is intended, not incidental, and bounded. With
      `--auto-confirm` live for all three declared agents, adding inbox topics means one
      `stage=delivered` receipt per topic per watermark ADVANCE — rung L2 of the arc-011
      receipt ladder (S5/S6), which is what AEF's T-3434 ladder is waiting on. Bounded by
      the durable per-topic offset guard at `_auto_confirm_topic` (`$notify_dir/.<agent>
      .<topic>.acked`), which returns early when `latest_off <= prev`, so 50 unread
      produces one receipt, not fifty. Verified by reading that guard, NOT by executing
      it against the live topic — see Findings F3 for what that leaves untested.

<!-- AC8 is deliberately NOT "AEF confirms receipt". Telling AEF the answer changed is
     gated on the prover, and — per Findings F1 — the prover does not cover this path. -->

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

**F1 — the prover cannot prove this path, and saying it did would repeat the bug.**
`scripts/notify-rail-e2e.sh:201` hardcodes `TOPIC="$(dm_topic "$SELF_FP" "$PEER_FP")"`.
There is no `--topic` override and no inbox shape anywhere in it. Run here it returned
**PROVEN, 5/5 stages** (PRECOND / DELIVER / RECEIPT / LADDER L3 / WAKE) — which is a real
regression check on the enumeration this task edited, and is NOT evidence about inbox
mail. The inbox evidence is the live before/after below, and it stops short of the full
chain. Reporting "prover PROVEN" as inbox proof would be the same error as the claim
corrected in T-3202.

**F2 — the fix is shipped but DARK (G-069 / T-2405 stale-code class).** All three
sidecars have been running since `Sep 22 14:16`, so they execute the pre-fix code. The
supervisor deliberately never kills a live sidecar (it only starts missing ones), so
nothing will pick this up on its own. Until each is restarted, `last_mail_ts` still does
not advance and AEF's 50 consults remain undelivered — the identical end state this task
set out to fix, from a different cause. **Restarting is an operator call, not an agent
one:** it posts an L2 receipt visible to AEF and it injects into live session prompts.

**F3 — what remains unproven.** Enumeration and counting are proven live (below).
Downstream of the flag the rail is topic-agnostic and the prover's WAKE stage passed, so
the chain is expected to complete — but "expected" is not "measured", and no inbox
message has yet traversed flag → injector → prompt end to end.

**Live before/after, same hub, pre-fix script vs working tree:**

    BEFORE  pending=44  last_mail_topic=dm:s3t2-1416551:d1993c2c3ec44c94
    AFTER   pending=94  last_mail_topic=inbox:cacc73ea32b121dd/010-termlink

The 50-message delta is AEF's unread consults. Premise checked before the edit, not
after: `channel unread --sender` returns 50 on that inbox topic against a dm control of
12, so the fix is not inert.

## Verification

bash tests/notify-sidecar-inbox-fixtures.sh
# AC1/AC3 — the inbox arm exists and is scoped to the self project, not a blanket prefix.
grep -q 'prefix "inbox:"' scripts/notify-sidecar.sh
# AC2 — declared constant with an env override, and NOT derived from the checkout path.
grep -q 'FW_SIDECAR_SELF_PROJECT' scripts/notify-sidecar.sh
test -z "$(grep -nE '^[^#]*basename' scripts/notify-sidecar.sh | grep 'SELF_PROJECT')"
# AC2 — both halves of the rail read the SAME self-identity variable. If these diverge,
# one half mirrors mail the other refuses to notice: the exact bug this task closes.
grep -q 'FW_SIDECAR_SELF_PROJECT' scripts/journal-mirror.sh
# AC4 — the dm: arm is still present and still keyed on the fingerprint (additive change).
grep -q 'prefix "dm:"' scripts/notify-sidecar.sh
# Syntax gate: this script runs detached under nohup from cron, where a parse error is
# invisible until the rail is silently dark again.
bash -n scripts/notify-sidecar.sh

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

### 2026-09-28T19:15:06Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3203-notify-sidecar-probes-dm-only-so-the-arr.md
- **Context:** Initial task creation
