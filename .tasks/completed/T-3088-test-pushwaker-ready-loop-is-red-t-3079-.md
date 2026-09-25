---
id: T-3088
name: "test-pushwaker-ready-loop is red: T-3079 rewrote the idle classifier and left
  its consumer's test stale"
description: >
  scripts/test-pushwaker-ready-loop.sh fails 3 assertions: 'ring returns 0 (rung at
  idle) expected 0 got 3', 'injected exactly once expected 1 got 0', 'inject fired
  only after READY (probe>=4) expected 4 got empty'. Cause is staleness, not a live
  break: lib/pty-state.sh was rewritten 2026-09-22 by T-3079 (quiescence + composer-row
  emptiness replacing UI-prose markers) and be-reachable-pushwaker.sh changed the
  same day by T-3069, while this test last changed 2026-09-09 under T-2933. Its shim
  still models the pre-T-3079 contract. Decide per assertion whether the test or the
  classifier is wrong before editing either: a test rewritten to match new behaviour
  proves nothing. Confirmed NOT caused by T-3086/T-3072 - that commit touched only
  notify-injector.sh, which is not in this test's dependency chain (it sources lib/pty-state.sh).

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [scripts/check-run-record-parse.sh, scripts/test-pushwaker-ready-loop.sh, tests/run-record-parse-fixtures.sh]
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
created: 2026-09-24T18:49:53Z
last_update: 2026-09-25T20:12:14Z
date_finished: 2026-09-25T20:12:14Z
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
  - ts: '2026-09-24T20:19:03Z'
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
  - ts: '2026-09-24T20:19:03Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius:
      tier: 2
      effort: 8
    rationale: blast_radius=? (no-components-UNMEASURED-not-zero); tier=2 
      (workflow:build); effort=8 (lines=204,acs=4)
    rubric_sha: e4a00f38e801
---

# T-3088: test-pushwaker-ready-loop is red: T-3079 rewrote the idle classifier and left its consumer's test stale

## Context

`scripts/test-pushwaker-ready-loop.sh` has been red since T-3079 rewrote the idle
classifier (`lib/pty-state.sh`, 2026-09-22: quiescence + composer-row emptiness replacing
UI-prose markers) while the test last changed 2026-09-09 under T-2933. Its shim still
models the pre-T-3079 contract.

**The governing constraint, from this task's own filing: a test rewritten to match new
behaviour proves nothing.** Each failing assertion has to be adjudicated — is the TEST
stale, or is the CLASSIFIER wrong? — before either is touched. The cheap move is to edit
the test until it passes, and that would convert a red test into a green one that asserts
whatever the code now happens to do. This whole session has been about checks that report
success without having verified anything; doing it here deliberately would be worse than
leaving it red.

## Acceptance Criteria

### Agent
- [x] Each of the three failing assertions is adjudicated INDIVIDUALLY and the verdict
      recorded with its evidence: *test stale* (the contract changed legitimately) or
      *classifier wrong* (the code regressed). A blanket "updated the test" is a failed
      criterion, not a shortcut.
- [x] The adjudication cites the actual T-3079 contract change — what `lib/pty-state.sh`
      now returns and why — rather than inferring the intended behaviour from what makes
      the assertion pass.
- [x] Whatever is wrong is FIXED in the place that is wrong. If an assertion encodes a
      real requirement the classifier no longer meets, the classifier is fixed; the
      assertion is only rewritten where the requirement itself legitimately changed.
- [x] `bash scripts/test-pushwaker-ready-loop.sh` exits 0 with every assertion passing,
      and the run is not made green by deleting, skipping or weakening an assertion —
      the assertion COUNT must not drop.
- [x] If any assertion cannot be honestly resolved in this task, it is left FAILING and
      recorded, not weakened. A partially-red suite with a written reason beats a green
      one that lies.

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

## Adjudication — per assertion, before either side was edited

All **four** failures (the filing recorded three; `deferred 3 times while BUSY — expected
'3' got '30'` was the fourth) trace to a **single** root, and the verdict is the same for
each: **the TEST was stale, specifically its environment shim. The classifier is right.**

| assertion | was | verdict |
|---|---|---|
| ring returns 0 (rung at idle) | got 3 | test stale — never reached READY, so the ring gave up |
| injected exactly once | got 0 | test stale — same cause |
| inject fired only after READY (probe>=4) | got empty | test stale — same cause |
| deferred 3 times while BUSY | got 30 | test stale — deferred all 30 attempts, same cause |

**The contract that changed (read, not inferred).** T-3079 replaced a one-sample,
marker-only verdict with a two-sample QUIESCENT one. `pushwaker_probe_pty`
(`lib/pty-state.sh:134`) now takes two tails a moment apart and calls
`pushwaker_quiescent_state`, which returns READY only when *all four* hold: not BUSY, not
modal, **`raw_a == raw_b`**, and **`composer-state.py` judges the composer row EMPTY**.

The shim modelled none of that. Its READY blob was a UI-prose footer
(`── **agent** ──> ⏸ ? for shortcuts`) carrying no `❯` glyph, so `composer_is_empty`
returned False and every verdict fell to UNKNOWN — deferring all 30 attempts. The blob is
still recognised by the *marker* path (`pushwaker_pty_state`, line 41), which is exactly
what made this look like a live break rather than staleness.

**Why "test stale" rather than "classifier wrong".** Two independent reasons, neither of
them "it passes now":

1. The T-3079 rewrite is documented and justified in-file — 269 paired samples across
   plain, long and tool-using turns, FALSE-READY = 0 — and it closes a named hole (during
   streaming, response text fills the tail and mid-stream samples showed NO busy marker, so
   marker-absence never meant idle).
2. **Every assertion keeps its original literal value** once the shim models the new
   contract — 0, 1, 4, 3, 3, 0, unchanged. If the requirements themselves had changed, the
   expected values would have had to move. They did not. The requirement ("wait while busy,
   inject exactly once the instant the REPL goes idle") was always right; only the model of
   the environment was obsolete.

**What changed in the test:** the shim scripts on a derived LOGICAL PROBE (two `pty output`
calls per probe) and returns identical bytes for both calls of one probe — a shim that
flipped mid-probe would make `a != b` and force UNKNOWN forever. Its READY blob is a real
composer frame with a bare `❯` (verified directly: `'❯ '` → exit 0, `'❯ hello'` → exit 1).
`PUSHWAKER_QUIESCE_DELAY=0` keeps it hermetic — a documented knob; both samples still
happen and still have to agree. No assertion was deleted, skipped or weakened.

## Mutation findings — and a real gap the mutation exposed

A test made to agree with the code proves nothing unless it can still go red, so both arms
were mutated:

- **`pushwaker_quiescent_state` → always READY** (the blind-ring regression): **CAUGHT**,
  4 assertions red including `always-busy NEVER injected (no blind ring)`.
- **`pushwaker_pty_busy` → never busy**: **NOT CAUGHT.** The suite passed with the BUSY
  detector entirely disabled.

The second one is the finding. The always-busy fixture emitted prose with no `❯`, so
`composer_is_empty` returned False and the verdict fell to UNKNOWN — the test was defending
the right behaviour *through the wrong arm*, and would have sat green through a real
regression in busy detection. The fixture now carries both the busy marker AND an empty `❯`
composer row, which isolates the BUSY arm: with it broken, the blob reads quiescent +
composer-empty = READY = a blind inject, and the assertion fires. **Re-verified: mutant now
CAUGHT (2 assertions red), and the unmutated tree still passes 6/6.**

## Observation — not acted on, recorded for whoever owns the rail

`pushwaker_pty_state` (the pre-T-3079 marker-only classifier, `lib/pty-state.sh:31`) has
**no production caller**. The only things referencing it are its own suite
(`test-pushwaker-filter.sh`, 8 cases, all passing) and a stale comment at
`lib-idle-gate.sh:18` that still tells readers the state comes from it.

That is the T-2699 shape — a covered, green, and uncalled surface, whose passing tests say
nothing about the rail actually in use. It is not deleted here: that is a scope decision for
whoever owns the waker, and this task's mandate was to adjudicate four assertions. But it is
the reason the diagnosis looked ambiguous, and the comment at `lib-idle-gate.sh:18` is
actively misleading about which function decides the verdict.

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
# Origin: T-1849/T-1730/T-1731 each added a legitimate hook without refreshing
# the baseline — FAIL sat for multiple sessions until T-1886 cleaned up.

# The suite is green and NO assertion was lost — the count is pinned at 6, so a future
# "fix" that deletes a failing assertion fails this line instead.
bash scripts/test-pushwaker-ready-loop.sh > /tmp/.t3088 2>&1 && grep -q "RESULT: PASS" /tmp/.t3088
test "$(grep -c '^ok: ' /tmp/.t3088)" = "6"

# The shim models the TWO-SAMPLE contract, asserted against the committed file (T-3086).
git show HEAD:scripts/test-pushwaker-ready-loop.sh > /tmp/.t3088-head 2>&1 && grep -q 'probe=$(( (n + 1) / 2 ))' /tmp/.t3088-head

# The READY blob carries the composer glyph the new classifier actually requires,
# and that glyph genuinely reads EMPTY to the helper (not asserted by eye).
grep -q '❯ ' /tmp/.t3088-head
printf '❯ ' > /tmp/.t3088-glyph && python3 scripts/lib/composer-state.py < /tmp/.t3088-glyph
printf '❯ hello' > /tmp/.t3088-glyph2 && ! python3 scripts/lib/composer-state.py < /tmp/.t3088-glyph2

# The always-busy fixture isolates the BUSY arm (the mutation finding).
grep -q "isolates the BUSY arm" /tmp/.t3088-head

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

### 2026-09-24T18:49:53Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3088-test-pushwaker-ready-loop-is-red-t-3079-.md
- **Context:** Initial task creation

### 2026-09-25T20:05:56Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-72a3eb1c
- **Timestamp:** 2026-09-25T20:12:16Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Per-AC findings:**

- **AC#2 (Agent)** — The adjudication cites the actual T-3079 contract change — what `lib/pty-state.sh`
  - **AC-verify-mismatch** (narrow, heuristic) — `path=lib/pty-state.sh in: The adjudication cites the actual T-3079 contract change — what `lib/pty-state.sh``

### 2026-09-25T20:12:14Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
