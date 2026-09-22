---
id: T-3079
name: "Idle classifier is blind to the current Claude Code UI, so the injector defers forever"
description: >
  scripts/lib/pty-state.sh classifies a live, idle, injectable Claude Code REPL as UNKNOWN, so notify-injector.sh defers rc=4 permanently and the L3 rung can never be reached by machine. Measured 2026-09-22: sampled every 10s for 100s after a completed turn, no positive marker at any sample. Cause is the marker set, not the window: ?forshortcuts is absent at every window size on a chat REPL with auto-mode on, whose footer reads 'auto mode on (shift+tab to cycle)'. Widening the window is NOT the fix — at 4000 bytes a stale esctointerrupt reappears and would pin BUSY forever, which is the scrollback contamination the docstring warns about. Also: the lib references $TERMLINK with no default, so a caller that has not set it reads nothing and gets UNKNOWN — a broken instrument indistinguishable from a real verdict.

status: work-completed
workflow_type: build
owner: claude-code
horizon: null
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
created: 2026-09-22T20:33:56Z
last_update: 2026-09-22T21:20:14Z
date_finished: 2026-09-22T21:20:14Z
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

# T-3079: Idle classifier is blind to the current Claude Code UI, so the injector defers forever

## Context

`scripts/lib/pty-state.sh` gates every injection on this rail. It classified a live,
idle, **injectable** Claude Code REPL as UNKNOWN — permanently — so `notify-injector.sh`
deferred at rc=4 and `be-reachable-pushwaker.sh` fell through to rc=3 after 90s of
patience. The L3 rung could never be reached by machine, which is why T-3069's live AC
failed twice and why SQ-2 concluded, wrongly, that the arc had no audience.

**Three defects, each measured, not inferred.**

1. **The marker set does not match the UI.** The READY arm required one of
   `?forshortcuts | newtask? | checkingforupdate | /cleartosave`. On a chat REPL with
   auto-mode on, `?forshortcuts` is ABSENT at every window size; that footer reads
   `⏵⏵ auto mode on (shift+tab to cycle)`. Sampled every 10s for 100s after a completed
   turn: no positive marker at any sample.

2. **The BUSY arm has a hole, and it is the dangerous one.** During streaming, response
   text fills the window and pushes `esc to interrupt` out of it. Five consecutive
   mid-stream samples reported NO busy marker. Marker absence never meant idle — it
   usually meant busy. Capturing a busy marker for a fixture took 25 rapid samples at
   turn *start* and could not be caught at all mid-stream.

3. **Widening the window makes it worse.** After a completed turn: 2500B no markers,
   4000B a **stale** `esc to interrupt` reappears, 6000B carries the stale busy marker
   AND idle markers together. That is precisely the scrollback contamination the file's
   own docstring warns about. The narrow window was right; the marker set was wrong.

**The fix does not depend on UI prose**, because prose is what broke it:

* **QUIESCENCE** — two reads a moment apart must be byte-identical. Every in-flight turn
  animates (spinner, elapsed seconds, token counter), so a running turn cannot hold
  still. This is what closes hole #2.
* **EMPTY COMPOSER** — the prompt's own row must hold nothing but whitespace and/or the
  UI's dim suggestion. This refuses to inject on top of text already pending, a second
  and distinct way to lose a message.

Both are structural properties of *how a terminal renders*, not of what it says.

**Validation:** 466 paired samples across plain, long and tool-using turns, scored
against the UI's own `· done` completion marker as ground truth. **FALSE-READY = 0**,
with every in-flight sample correctly deferred.

Cost of the remaining false-negative: ~4% of idle samples defer because a transient
status line repaints. That is safe and self-correcting — a missed wake costs one cron
cycle; a blind inject costs the message.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] **The fail-safe bias is preserved and provable.** READY is still returned ONLY on
      positive evidence; every ambiguous state resolves to UNKNOWN, and the
      BUSY → modal-UNKNOWN → READY case ORDER is unchanged. A wrong READY is a blind
      inject (T-2396), so this is the criterion the others serve
- [x] **A live, idle, injectable REPL classifies READY** — the state that currently
      returns UNKNOWN forever. Proven on a real spawned REPL, not on a fixture alone
- [x] **A mid-turn REPL never classifies READY.** Proven by sampling a real REPL
      repeatedly ACROSS a running turn, not by a single well-timed probe
- [x] The READY predicate does not rest on UI *prose* alone. Footer wording changed
      between Claude Code versions and that is what broke this classifier; the new
      signal must survive a wording change or the same bug recurs next release
- [x] **A composer with text already pending does NOT classify READY**, because
      injecting there appends to the operator's half-typed line — a distinct way to
      lose a message that the current classifier also cannot see
- [x] `TERMLINK` resolves to a default inside the lib, so a caller that has not
      exported it can no longer get a silent UNKNOWN from an unrunnable probe. The
      broken-instrument case must be distinguishable from a real verdict
- [x] Fixtures are built from **PTY bytes captured off a real REPL**, not hand-written
      approximations, covering: idle, mid-turn, text-pending, modal/picker, shell
      prompt, and empty read. Each asserts the classifier's verdict
- [x] **Mutation-proven:** disabling each new arm of the predicate individually makes a
      named fixture fail. A guard that cannot go red is not a guard
- [x] `bash tests/notify-injector-fixtures.sh` still passes in full — the injector
      sources this lib, and no existing assertion may be weakened to fit.
      (AMENDED: written as "23/23"; it is now **24/24**. The extra case is T24, added
      because this task found the injector reporting "L3 posted" for a receipt that
      was never written. The count went up, not down — no assertion was removed.)

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

bash -n scripts/lib/pty-state.sh
python3 -m py_compile scripts/lib/composer-state.py

# The classifier's own verdicts, every fixture real captured PTY bytes. F1 is the
# state that was UNKNOWN forever; F3/F4/F5/F5b are the four ways a message is lost.
bash tests/pty-state-fixtures.sh

# The injector sources the lib. T24 pins the false "L3 posted" claim found live.
bash tests/notify-injector-fixtures.sh

# Structural pins, so a later edit cannot quietly drop the arms the fixtures assert.
grep -q 'pushwaker_composer_empty' scripts/lib/pty-state.sh
grep -q 'raw_a" = "$raw_b' scripts/lib/pty-state.sh
grep -q 'TERMLINK="${TERMLINK:-termlink}"' scripts/lib/pty-state.sh
grep -q 'l3_covered' scripts/notify-injector.sh
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

### 2026-09-22 — the arc's blocker was one function, and the search for it was nearly sabotaged by a second bug in the same file

- **What changed:** filed as "the marker set is stale; add markers". That would have
  fixed nothing. Adding `automodeon` to the list gets you READY for about as long as
  the next Claude Code release, and it does not touch the far more dangerous defect the
  measurement surfaced: **the BUSY arm has a hole.** During streaming the response text
  pushes `esc to interrupt` out of the window, so five consecutive mid-stream samples
  reported "not busy". The classifier's safety therefore never rested on the BUSY
  marker in the first place — it rested on READY being unreachable. Fix the READY arm
  alone and you convert a permanently-deferring classifier into one that injects into
  running turns.
- **Plan impact:** the predicate had to stop depending on UI prose at all. Quiescence
  (two byte-identical reads) and composer-row emptiness are properties of how a
  terminal repaints, not of what it writes, so a wording change in the next release
  cannot silently disarm them.
- **The measurement instrument was itself broken, twice, and both nearly produced
  confident wrong answers:**
  1. `pty-state.sh` referenced `"$TERMLINK"` with no default. Sourced from a shell that
     had not exported it, the probe ran an empty command, read nothing, and returned
     UNKNOWN — indistinguishable from a real verdict. My first sweep of five sessions
     reported all-UNKNOWN *through this bug* and would have "confirmed" SQ-2 on a
     measurement error.
  2. My first composer parser judged "everything after the last `❯`" in the byte
     stream. The PTY is cursor-addressed, so byte order is not screen order: the
     version/update status line (`current: 2.1.267 … Checking for update`) is grey but
     NOT dim and is painted at row 19 while the composer sits at row 21. It counted as
     pending input, and the classifier deferred forever on an idle REPL — the same
     "never READY" bug reintroduced one layer down, by the fix for it. Only row
     tracking resolved it.
- **Mutation testing earned its keep twice, and both times the suite was green and
  wrong.** First run: deleting the quiescence arm broke nothing, because my mid-turn
  fixture was being caught by the composer arm instead. The state that isolates
  quiescence — in flight, no busy marker, composer genuinely empty — had to be hunted
  live and captured (F5b). Second run: T24 passed with the read-back disabled, because
  the stub returned an empty topic and a different branch fired. A stub that models a
  real hub (never empty) fixed it. Both are the same lesson: a fixture that passes for
  the wrong reason is indistinguishable from one that passes for the right one, until
  you break the code on purpose.
- **A defect found only because the live proof was actually run.** The injector
  reported `L3 posted (evidence=idle-gated-inject)` for a receipt that did not exist.
  `notify-ack-read`'s contract is explicit — *"Exit: 0 posted (OR already acked)"* — so
  with the guard at 145 and the queue serving offset 0 it took the no-op branch and
  returned 0. This file already carried a scar from the same class arriving by another
  route (a stale guard in the wrong directory); that fix addressed the route and left
  the conflation, which is exactly why it recurred. The injector now READS THE RECEIPT
  BACK from the hub and refuses to claim delivery it cannot see. Disavowed evidence
  (`wake-consumer`) does not count toward that proof.
- **Still open, recorded rather than papered over:** the queue picks the oldest content
  message on the topic with no watermark tied to the L3 rung, so on a topic with
  history it re-serves ancient offsets. On the live run it served offset 0 while L3
  stood at 145. Not fixed here — it is a queue-semantics change, not a classifier fix,
  and it deserves its own task.
- **Triggered:** `tests/pty-state-fixtures.sh` (new, 12 assertions, fixtures from real
  captured bytes), `scripts/lib/composer-state.py` (new), T24 in the injector suite,
  and the read-back verification in `notify-injector.sh`. Closes T-3069's live AC.

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

### 2026-09-22T20:33:56Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3079-idle-classifier-is-blind-to-the-current-.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-ba376943
- **Timestamp:** 2026-09-22T21:20:17Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-09-22T21:20:14Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Classifier fixed with two prose-independent arms (quiescence + composer-row emptiness); 466 paired samples, FALSE-READY=0; 12 fixtures from real captured PTY bytes; all arms mutation-proven. Also fixed the injector claiming L3 posted for a receipt that was never written.
