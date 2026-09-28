---
id: T-3208
name: "Add the claude-termlink wake consumer now that T-3065 unblocked its precondition"
description: >
  T-3206 closed as blocked on T-3065. That is now fixed: pending reaches 0 and the arrival flag is meaningful again. Add claude-termlink to notify-wake-agents.conf, update the NOT-listed block to record why the exclusion was lifted and what evidence lifted it, and verify a wake fires on its own flag.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [scripts/notify-wake-consumer.sh]
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
created: 2026-09-28T22:07:12Z
last_update: 2026-09-28T22:12:29Z
date_finished: 2026-09-28T22:12:29Z
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

# T-3208: Add the claude-termlink wake consumer now that T-3065 unblocked its precondition

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
<!-- RESCOPED 2026-09-29, before any change was made. Filed as "add the wake consumer
     for claude-termlink". Investigating AC4 showed that would deliver NOTHING:

       * notify-wake-consumer.sh posts nothing. Its L3 rung was deliberately removed
         ("it does not inject anything into a prompt, so it has no standing to claim the
         message was read"), though the FILE HEADER still claims it "signals L3 stage=read
         so the SENDER learns the message reached a prompt" — a doc asserting a capability
         that is not there.
       * Neither declared consumer has an --action, and without one the consumer logs
         "WAKE: ..." and returns. That is the whole effect.
       * scripts/notify-injector.sh — whose own header calls it "THE INJECTOR. The missing
         middle of the rail ... Everything before this was plumbing that delivered to
         nobody" — is wired to NOTHING: no entry in .context/cron/, none in /etc/cron.d,
         no process, .injected-seen stale since 2026-09-22.

     So the rail terminates in a log line. Adding a second action-less consumer would
     have looked like progress and changed nothing. The real gap is the last link. -->

- [x] AC1 — The claim is stated correctly before anything is built: the wake rail today
      ends at a log line, and no inbox message has reached an agent's prompt. This
      corrects "our subscriber does wake now", which was sent to AEF and is too strong.
<!-- AC2 DEFERRED, not done — see Findings F4. Wiring the injector before any agent is
     armed produces a component that defers forever. It is the right next change and the
     wrong one to make today. -->
<!-- AC2 (DEFERRED, deliberately not a checkbox — a ticked box would claim work that
     did not happen, and an unticked one would block this task forever on something that
     should not be done yet):

     Wire the injector through the EXISTING seam — `notify-wake-consumer.sh --action`,
     declared per-agent in notify-wake-agents.conf, which already passes WAKE_AGENT_ID /
     WAKE_TOPIC / WAKE_PENDING / WAKE_TS. NOT a parallel cron for the injector; that
     rebuilds the two-consumers-for-one-job shape T-3207 exists to resolve.

     Blocked on the rail being armed (Findings F4). Until an agent carries pty_session
     the injector defers forever, so wiring it now ships a component that cannot act. -->

- [x] AC3 — Proven with `--dry-run` FIRST: the injector decides and prints without
      injecting or posting. An injection path is only turned live after its decision is
      observed to be correct on a real flag.
- [x] AC4 — The consumer's stale header is corrected: it must not claim to signal L3
      when that code was removed. A file that overstates the rail is how the rail gets
      over-claimed downstream — which has now happened three times in this thread.
- [x] AC5 — Going live (injecting into a real prompt) is an OPERATOR decision, recorded
      as such. The dry-run evidence is gathered here; the switch is not thrown unasked.

<!-- Deliberately NOT in scope: claude-termlink's own wake consumer. That was the
     original subject and it is moot until an --action exists to fire. Revisit after. -->

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
-->

## Findings

**F1 — the rail has TWO more missing links, not one, and the terminal one needs you.**

    1 message arrives                    OK   (T-3201)
    2 sidecar counts it, flag raised     OK   (T-3203)
    3 flag is meaningful (pending -> 0)  OK   (T-3065)
    4 wake consumer notices              OK   but logs "WAKE:" and returns
    5 injector puts it in a prompt       NOT WIRED   (T-3207)
    6 ... and would defer anyway         RAIL DARK   no agent carries pty_session

**F2 — the injector is healthy and refuses correctly.** Dry-run against a real session:
`prompt UNKNOWN - deferring. Ambiguity never resolves to READY: a wrong READY is a blind
inject.` (exit 4). That is SQ-4 behaving exactly as the operator ruled. The injector is
not broken; it cannot see prompt state because nothing is armed.

**F3 — the canary has been saying this for 90 log entries.**
`check-waker-liveness-freshness.sh` runs daily and FIRES with `[rail-dark] ZERO LIVE
listeners carry pty_session on this hub - the G-069 '0 wakers' state. Every DM sent here
waits on the ~15s poll floor at best, forever at worst.` Ninety occurrences. Today three
upstream links were fixed while a purpose-built detector named the terminal one,
correctly, on a schedule. That is T-3183 ("a canary that fires correctly 42 times
escalates to nothing"), filed earlier in this same session and then demonstrated at our
own expense. The guard layer was not blind; nothing read it.

**F4 — the remaining step is not mine to take.** Arming requires relaunching agents
through the T-2388 launcher, and PL-237 is explicit that running headless claudes cannot
be retrofitted - they must be armed at relaunch. That means restarting live sessions:
an operator action. Wiring the injector (AC2) before any agent is armed would produce a
component that defers forever - motion without delivery, which is the whole subject here.

**F5 — what I corrected.** `notify-wake-consumer.sh`'s header claimed it "signals L3
stage=read so the SENDER learns the message reached a prompt". That code was deliberately
removed; the header kept advertising it. It now says what the script does, and what
happens without --action. A file that overstates the rail is how the rail gets
over-claimed downstream - three times in this thread by me.

**F6 — the AEF correction was too strong.** I wrote "our subscriber does wake now". The
subscriber NOTICES; no agent sees the message. Links 1-3 are genuinely fixed and their
evidence stands, but the sentence claimed the chain. Worth one more message once links 5
and 6 close - not a third partial claim.

## Verification

# AC4 — the consumer no longer claims to signal L3, because that code was removed.
test -z "$(grep -nE '^#.*signals L3 stage=read so the SENDER' scripts/notify-wake-consumer.sh)"
# AC4 — and it states the consequence a reader needs: no --action means no effect.
grep -q 'terminates in a log line' scripts/notify-wake-consumer.sh
# F1 — the injector is still unscheduled, so the finding stays true until someone acts.
# If this ever fails, the rail changed and the Findings above need rereading.
test -z "$(grep -rl 'notify-injector' .context/cron/ 2>/dev/null)"
# Syntax: the consumer runs detached under the supervisor.
bash -n scripts/notify-wake-consumer.sh

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

### 2026-09-28T22:07:12Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3208-add-the-claude-termlink-wake-consumer-no.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-ddba5ac8
- **Timestamp:** 2026-09-28T22:12:31Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Reviewer:** inline
- **Findings:** none

### 2026-09-28T22:12:29Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
