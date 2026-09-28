---
id: T-3206
name: "claude-termlink holds the inbox mail but has no wake consumer of its own"
description: >
  T-3204 F5. claude-termlink carries pending=95 and last_mail_topic=inbox:... but
  notify-wake-agents.conf declares no consumer for it; its wake is driven by claude-termlink-alt's
  consumer running --as-identity claude-termlink. alt auto-confirms the inbox mail
  before its own flag records the arrival, so alt's flag shows pending=0 and a stale
  dm topic and its wake-seen is still Sep 22. The flag with the signal and the flag
  that drives the wake are different files.

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
created: 2026-09-28T20:43:32Z
last_update: 2026-09-28T21:31:15Z
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
  - ts: '2026-09-28T21:31:16Z'
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

# T-3206: claude-termlink holds the inbox mail but has no wake consumer of its own

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
<!-- RESCOPED before any work. This task was filed as "claude-termlink has no wake
     consumer — add one". That premise is WRONG and the record says so plainly.
     `.context/cron/notify-wake-agents.conf` carries an explicit NOT-listed block:

       claude-termlink — its flag carries 91 pending from stale July topics, so a
                         consumer would fire immediately and continuously on
                         history rather than on new mail. Sweep that backlog
                         first; a wake that fires on everything wakes nobody.

     So the exclusion is deliberate, reasoned, and carries a stated precondition.
     Adding the config line is not the fix — it is the thing the author refused to
     do, for a reason that still holds. Same class as T-3200: the record already
     contained the answer. -->

- [x] AC1 — The PRECONDITION is measured, not assumed: a per-topic breakdown of
      claude-termlink's pending, separating genuinely-unread current mail from the
      stale July test residue (`s3g`, `s3probe`, `s3smoke`, `s3t*` and similar) the
      conf names. Without that split there is no way to tell a real wake from history.
- [x] AC2 — The structural blocker is named and addressed or explicitly deferred:
      under T-3065 the auto-confirm receipt is signed with the per-agent key rather
      than the `--self-fp` party, so claude-termlink's unread NEVER reaches 0, so
      `pending` is permanently >0, so `last_mail_ts` re-advances EVERY cycle. A
      consumer added on top of that fires forever regardless of how clean the backlog
      is. Sweeping alone does not satisfy the conf's precondition while T-3065 stands.
- [x] AC3 — Any backlog sweep is non-destructive to other readers. `inbox:` topics are
      a SHARED project mailbox with multiple readers; acking on our own cursor is
      fine, trimming the topic is not. No `channel trim` / `inbox clear` on a shared
      topic as part of this.
- [x] AC4 — The conf's NOT-listed block is UPDATED, not silently deleted, if
      claude-termlink is eventually added: the next reader must be able to see why the
      exclusion was lifted and what evidence lifted it.
- [x] AC5 — If the honest outcome is "cannot add the consumer until T-3065 is fixed",
      this task says that and stops, rather than shipping a config line that produces
      a continuously-firing wake. A wake that fires on everything wakes nobody.

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

**F1 — the exclusion is deliberate and its stated reason has DECAYED.**
`notify-wake-agents.conf` excludes claude-termlink because "its flag carries 91 pending
from stale July topics". Measured today, the 95 pending are not that:

      50  inbox:cacc73ea32b121dd/010-termlink        <- AEF's real consults
      13  dm:3bba15e681b3a078:...                    <- framework-agent-systemd, real
      12  dm:d1993c2c3ec44c94:fd794e5408011572       <- real peer
       5  dm:d1993c2c3ec44c94:d1993c2c3ec44c94       <- self-DM
       4  dm:9219671e28054458:...                    <- real peer
       4  dm:cashweb-integration-agent:...           <- real peer
       3  dm:8e6fd77ec6f74b37:...                    <- real peer
       1  dm:acp-probe:...            \
       1  dm:...:deadbeefdeadbeef      >  4 total = the actual stale residue
       1  dm:s3t1-1416551:...         /
       1  dm:s3t2-1416551:...        /

The July test residue the conf names is **4 messages, not 91**. The backlog today is
dominated by genuine unread mail. So the premise that a consumer would "fire on history
rather than new mail" no longer describes reality — it would mostly fire on real mail.

**F2 — but the exclusion should STAND, for a reason the conf does not give.**
Sweeping the backlog is the conf's prescribed precondition and it would not work.
claude-termlink's sidecar recorded an auto-confirm guard at offset 49 on the inbox
topic, yet `channel unread --sender d1993c2c3ec44c94` on that same topic still returns
**50**, and the only two receipts on it carry senders `3bba15e681b3a078` and
`6738c073bbcc587a` — never `d1993c2c3ec44c94`. The ack does not register under the
identity whose cursor is being measured. That is T-3065 observed directly.

Consequence: claude-termlink's `pending` can never reach 0, so `last_mail_ts` re-advances
every cycle (`write_cycle`: `if pending > 0 then mail_ts=hb`), so a consumer added now
fires every cycle forever — bounded only by the 30s per-topic cooldown. Exactly the
"wake that fires on everything wakes nobody" outcome the conf was protecting against,
reached by a different route.

**F3 — therefore, per AC5, this task stops rather than ships a config line.**
The honest state: **T-3206 is blocked on T-3065**, not on a backlog sweep. Anyone who
follows the conf's advice will sweep, watch pending return, and lose time to a
correct-sounding instruction whose premise expired. The conf's NOT-listed block should
be corrected to say so (AC4) — that is the deliverable here, not a consumer.

Worth stating plainly: I filed this task myself, hours ago, as "add the missing
consumer". Reading the config I was about to edit is what stopped a wrong fix.

## Verification

# AC5 — claude-termlink is still NOT declared. This task deliberately shipped no
# consumer; if this line ever fails, someone added it without fixing T-3065 first.
test -z "$(grep -v '^#' .context/cron/notify-wake-agents.conf | grep -v '^$' | grep '^claude-termlink ')"
# AC5 — behaviour unchanged: exactly the two consumers that were declared before.
test "$(grep -v '^#' .context/cron/notify-wake-agents.conf | grep -vc '^$')" -eq 2
# AC4 — the corrected blocker is recorded where the next reader will look.
grep -q 'T-3065' .context/cron/notify-wake-agents.conf
# AC4 — the ORIGINAL reason is preserved, not silently overwritten.
grep -q '91 pending from stale July topics' .context/cron/notify-wake-agents.conf
# AC3 — no sweep was performed, so the shared mailbox is intact: AEF's consults are
# still on the topic for every other reader. Asserts the RECORD COUNT on the live
# topic, not a property of a local file — a trim is exactly what would reduce it.
test "$(TERMLINK_RUNTIME_DIR=/var/lib/termlink termlink channel info 'inbox:cacc73ea32b121dd/010-termlink' --json 2>/dev/null | jq -r '.count // 0')" -ge 50

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

### 2026-09-28T20:43:32Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3206-claude-termlink-holds-the-inbox-mail-but.md
- **Context:** Initial task creation

### 2026-09-28T21:31:15Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
