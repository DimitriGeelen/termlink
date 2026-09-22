---
id: T-2975
name: "Fix /canaries FIRING-to-HEALTHY masking predicate + fixture"
description: >
  S-1/C-13: canary-status.sh:20-21 keys FIRING on log-mtime >= heartbeat-mtime; an
  ad-hoc run refreshes heartbeat without appending, flipping FIRING to HEALTHY. Fix
  predicate (fire on unacknowledged log content, heartbeat never clears) + fixture
  pinning it. Evidence: docs/reports/VALUE-REVIEW-repo-2026-09-19-consolidated.md
  C-13.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [value-review, arc:arc-009]
components: [scripts/canary-status.sh, tests/canary-status-firing-fixtures.sh]
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
created: 2026-09-19T21:57:55Z
last_update: 2026-09-20T19:07:38Z
date_finished: 2026-09-20T19:07:38Z
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
  - ts: '2026-09-20T08:45:09Z'
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
  - ts: '2026-09-20T08:45:19Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius:
      tier: 2
      effort: 8
    rationale: blast_radius=? (no-components-UNMEASURED-not-zero); tier=2 
      (workflow:build); effort=8 (lines=207,acs=4)
    rubric_sha: e4a00f38e801
---

# T-2975: Fix /canaries FIRING-to-HEALTHY masking predicate + fixture

## Context

The filing (S-1/C-13) is right that the masking exists and wrong about the remedy.

**Measured (AC1/AC2), 30 canaries on the real tree:** 25 healthy, 3 firing, 2 not-scheduled.
All three FIRING rows have `log_mtime == heartbeat_mtime` to the second. Seven canaries hold
non-empty logs; four of those read HEALTHY. Six of the non-empty-log canaries carry the
signature of genuine resolution — heartbeat and log share the **same minute-of-day** weeks
apart (dead-letter 07:39/07:39, fleet-doorbell-mail 09:23/09:23, stuck-claims 07:27/07:27),
which is a fixed-time daily cron that has re-evaluated and appended nothing since.

**The mechanism is sharper than filed.** The two writes have different authors: the check
SCRIPT touches its own `.heartbeat`, while the LOG is appended by the *crontab's* `>>`
redirect (`check-stuck-claims-freshness.sh:44` vs `.context/cron/stuck-claims-canary.crontab`).
So running `bash scripts/check-<x>.sh` by hand sends findings to the terminal, appends
nothing, and advances the heartbeat. A FIRING canary reads HEALTHY from that moment on —
and an ad-hoc run is exactly what this file prescribes as the operator's response to a
firing canary. That is a real, non-synthetic masked path, and it satisfied AC3's bar.

**But the predicate cannot be fixed, and the filed fix would do harm.** Genuine resolution
and ad-hoc masking leave *byte-identical* state: `log non-empty, heartbeat newer`. No
mtime-only predicate can separate them. The filing's prescription — "fire on unacknowledged
log content, heartbeat never clears" — would hold framework-pickup (63,993 B),
stale-waker-code, stuck-claims and fleet-doorbell-mail permanently red over resolved
history: the T-2818/T-2833 fatigue trap traded for the masking one. The existing fixture
suite already warned about precisely this ("If that leg ever fails, the fix has turned every
canary that ever fired into a permanent red light").

**Shipped instead:** the status taxonomy, exit codes and JSON `status` values are unchanged
— a new status is an ungated contract change to a layer other tools parse — and the reader
stops over-claiming. `SCOPE_NOTE` is defined once and printed on the full render, the
quiet-with-problems path and the JSON envelope, stating that HEALTHY over a non-empty log
means "no NEW entry since the last heartbeat", never "resolved". The `else`-branch comment
now names both causes instead of asserting only resolution.

**Two paths deliberately carry no scope line:** quiet-with-no-problems (emits nothing by
design — adding output there would break every cron consumer) and the zero-canaries path
(no log exists to be misread; that is a no-data condition, not a health report).

**Side findings, not fixed here:** (a) `/canaries` prints a log's signal-bearing line
directly beneath a HEALTHY row, so an operator sees `verdict=setup-fail` under a green
status — observed live on fleet-doorbell-mail, and it broke this task's own first fixture
attempt; (b) heartbeat format is inconsistent — ~6 canaries write an ISO timestamp, the rest
are bare `touch`, so the content slot needed to record invocation provenance already exists
on some and not others. The real fix — have each check script refuse to touch its heartbeat
when stdout is a terminal (`[ -t 1 ]`), or record provenance in it — spans 27 scripts and is
its own task.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] AC1: The mechanism is MEASURED on the real tree, not asserted. For every canary
      holding a non-empty log, record log_size / log_mtime / heartbeat_mtime and the
      classification `classify()` produces, and state how many read HEALTHY while
      holding unacknowledged log content.
- [x] AC2: Each HEALTHY-with-content canary is separated into RESOLVED (a later run
      re-evaluated the SAME condition with the SAME scope and found nothing — the
      documented and correct semantics of the `else` branch) versus MASKED (the
      heartbeat was advanced by a run that did not re-evaluate that condition:
      narrower flags, different scope, or an inspection run). Counts for both, with
      the evidence that distinguishes them.
- [x] AC3: A predicate change ships ONLY if AC2 yields at least one genuine MASKED
      instance on the real tree, or a non-synthetic path to one. If AC2 yields zero,
      that is recorded and NO predicate change ships — a fix justified only by a
      fixture it was written to satisfy is the T-2831 vacuous-check class, and
      "permanently FIRING until acknowledged" is the T-2818/T-2833 fatigue trap in
      the opposite direction.
- [x] AC4: Whatever ships (including a decision to ship no predicate change),
      `/canaries` states on every output path what its green does and does not
      cover, per the T-2680 scope-disclaimer precedent.
- [x] AC5: `tests/canary-status-fixtures.sh` (or the suite that covers this script)
      passes before and after. Any new assertion carries a mutant leg that fails
      when the change is reverted — a guard's green is not evidence until it has
      been fed the violation it claims to catch (PL-328).

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
# Origin: T-1849/T-1730/T-1731 each added a legitimate hook without refreshing
# the baseline — FAIL sat for multiple sessions until T-1886 cleaned up.

bash tests/canary-status-firing-fixtures.sh > /tmp/.t2975-fix 2>&1 && grep -q "failed: 0" /tmp/.t2975-fix
bash tests/canary-status-firing-fixtures.sh > /tmp/.t2975-fix2 2>&1 && grep -q "passed: 12" /tmp/.t2975-fix2
bash tests/canary-status-worktree-fixtures.sh > /tmp/.t2975-wt 2>&1 && grep -q "0 failed" /tmp/.t2975-wt
bash tests/canary-log-isolation-fixtures.sh > /tmp/.t2975-iso 2>&1 && grep -q "0 failed" /tmp/.t2975-iso
test "$(grep -c '^SCOPE_NOTE=' scripts/canary-status.sh)" = "1"
test "$(grep -c 'scope: \$SCOPE_NOTE' scripts/canary-status.sh)" = "2"
grep -q '"scope":"%s"' scripts/canary-status.sh
grep -q "masked path" tests/canary-status-firing-fixtures.sh
grep -q "mutant-scope" tests/canary-status-firing-fixtures.sh
bash scripts/canary-status.sh > /tmp/.t2975-real 2>&1 || true
grep -q "scope: HEALTHY on a non-empty log" /tmp/.t2975-real
bash scripts/canary-status.sh --json > /tmp/.t2975-json 2>/dev/null || true
python3 -c "import json; d=json.load(open('/tmp/.t2975-json')); assert d['scope']"
test -z "$(git status --porcelain .agentic-framework/)"

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

**Symptom:** `/canaries` reports HEALTHY for a canary whose log holds unresolved findings.
Concretely: a canary that is FIRING flips to HEALTHY the moment anyone runs its check
script by hand, and stays that way. Observed live — fleet-doorbell-mail renders
`verdict=setup-fail` directly beneath a green HEALTHY row.

**Root cause:** the heartbeat and the log have *different authors*. The check script
touches its own `.heartbeat`; the log is appended by the crontab's `>>` redirect. So any
invocation not wrapped by that crontab — an operator's ad-hoc run, a differently-flagged
run — advances the heartbeat while writing nothing to the log. `classify()` decides
HEALTHY-vs-FIRING purely from `log_mtime` vs `heartbeat_mtime`, and that ordering has two
causes it cannot tell apart: genuine resolution, and an un-redirected run.

**Why structurally allowed:** the reader infers "resolved" from an ordering that does not
entail it, and nothing anywhere records *which kind of run* produced a heartbeat. T-2826
hardened the equality case (`-gt` → `-ge`) and deliberately preserved the
historical-stays-HEALTHY branch as correct, which pinned the remaining half as intended
behaviour. The masking is therefore invisible to every existing check, and the documented
operator response to a firing canary ("Ad-hoc check: `bash scripts/check-<x>.sh`") is the
action that triggers it.

**Prevention:** the reader no longer claims more than it knows — `SCOPE_NOTE` on all three
reporting paths states that HEALTHY over a non-empty log means "no NEW entry since the last
heartbeat", never "resolved" — plus fixture case 6, which reproduces the masking sequence
(FIRING → advance heartbeat only → HEALTHY) so any future predicate change must confront it,
and two mutants proving the declaration is load-bearing. This is detection and honesty, not
a cure: the cure is provenance in the heartbeat (`[ -t 1 ]`, or a recorded invocation
marker), which spans 27 check scripts and is filed as its own task rather than smuggled in
here.

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

- **The filing's premise held; its prescribed remedy did not.** "Fix predicate (fire on
  unacknowledged log content, heartbeat never clears)" would have put framework-pickup,
  stale-waker-code, stuck-claims and fleet-doorbell-mail permanently red over history that
  six independent daily cron runs had already resolved. Shipping it would have traded the
  masking defect for the T-2818/T-2833 fatigue defect and called it a fix. Third task this
  run where measuring the recorded premise changed the work.

- **The two states are byte-identical, so no predicate could have worked.** This was not
  discoverable from the filing, only from reading who writes the heartbeat versus who writes
  the log. It is the T-2875/T-2876 shape again — outcomes indistinguishable from one side —
  arriving in the monitoring layer rather than the comms layer.

- **The existing fixture suite had already written the warning.** Its header says "If that
  leg ever fails, the fix has turned every canary that ever fired into a permanent red
  light." T-2826 anticipated exactly the over-correction this task was asked to make. The
  guard layer's own prose was the best evidence available, and it was free.

- **A new status was the tempting fix and was declined.** `UNVERIFIED` as a fourth,
  non-firing class would genuinely stop the false assurance — but `status` flows into the
  JSON envelope, the exit-code counters and the meta-canary, and changing it is an ungated
  contract change to a layer other tools parse. Recorded as the follow-up, not opened here
  ("one lock at a time").

- **The first fixture attempt failed, and the failure was the finding.** Asserting on
  rendered text matched the word FIRING inside the planted *log body*, because the script
  prints a log's signal-bearing line beneath a HEALTHY row. Re-anchoring the assertion on
  the JSON status field fixed it — and surfaced side finding (a), which is a real operator
  hazard nobody had filed.

- **Cost was dominated by four gates, not by the edit.** P-002 refused a `for` loop and a
  leading `VAR=` assignment; G-020 refused the read-only `canary-status.sh` run and the
  estimator. Reaching a cost score required passing four gates in sequence, and the last of
  them required writing the ACs first — so the estimator's `acs=`/`lines=` inputs are a
  function of work the gate forced to happen before the estimate.

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

### 2026-09-19T21:57:55Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2975-fix-canaries-firing-to-healthy-masking-p.md
- **Context:** Initial task creation

### 2026-09-19T22:08:33Z — status-update [task-update-agent]
- **Change:** tags: +arc:arc-009

### 2026-09-20T18:57:10Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-a9395ec3
- **Timestamp:** 2026-09-20T19:07:44Z
- **Catalogue:** v1.3-seed
- **Overall:** FAIL
- **Needs Human:** no
- **Findings:** 3

**Per-AC findings:**

- **AC#5 (Agent)** — AC5: `tests/canary-status-fixtures.sh` (or the suite that covers this script)
  - **AC-verify-mismatch** (narrow, heuristic) — `path=tests/canary-status-fixtures.sh in: AC5: `tests/canary-status-fixtures.sh` (or the suite that covers this script)`

**Verification-level findings:**

  1. **swallowed-errors** (severe, deterministic) @ Verification:line 69
     - evidence: `bash scripts/canary-status.sh > /tmp/.t2975-real 2>&1 || true`
  2. **swallowed-errors** (severe, deterministic) @ Verification:line 71
     - evidence: `bash scripts/canary-status.sh --json > /tmp/.t2975-json 2>/dev/null || true`

### 2026-09-20T19:07:38Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
