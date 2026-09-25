---
id: T-3145
name: "Axis B omits exec — the one invocation form that needs the exec bit, and the one that is dead"
description: >
  Axis B omits exec — the one invocation form that needs the exec bit, and the one that is dead

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [scripts/check-framework-tracking-drift.sh, tests/framework-dangling-ref-fixtures.sh]
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
created: 2026-09-25T14:08:56Z
last_update: 2026-09-25T14:16:12Z
date_finished: 2026-09-25T14:16:12Z
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

# T-3145: Axis B omits exec — the one invocation form that needs the exec bit, and the one that is dead

## Context

Axis B of `check-framework-tracking-drift.sh` (T-2817) exists to catch exactly one thing:
*"tracked framework code SOURCES or EXECUTES a `"$FRAMEWORK_ROOT/<path>"` that does not
resolve."* Its anchor at line 226 matches `.` · `source` · `bash` · `sh` · `python3` ·
`python`.

**It does not match `exec`** — and `exec` is the only one of those forms that requires the
target to be executable. `bash foo.sh` runs a mode-644 file happily; `exec foo.sh` returns
126 Permission denied. So the guard omits precisely the verb where file mode is
load-bearing.

Measured 2026-09-25 (T-3143): there are exactly three exec-position references in the
vendored tree, and all three are outside the candidate set.

    exec "$FRAMEWORK_ROOT/bin/watchtower.sh"    ok
    exec "$FRAMEWORK_ROOT/lib/build.sh"         DEAD — 126 Permission denied
    exec "$FRAMEWORK_ROOT/metrics.sh"           ok

`lib/build.sh` is vendored at mode 100644, so `fw build` — advertised at `fw help` line 85
— cannot run. The repo scans clean by every existing test: axis A sees a tracked file, a
mode-drift comparison sees index 100644 agreeing with disk 644, and axis B never examines
the reference at all. Filed upstream at `framework:pickup` offset 162; not patched here
because `lib/build.sh` is vendored (G-062). **This task builds the local detector**, which
is the half that survives a re-vendor — the same split as T-2859 and T-2833.

The existence test is not enough on its own and that is the whole point: `-e` passes on
`lib/build.sh` today. The new class is *resolves but cannot execute*, which is a different
errno (126, not 127) and a different remediation (chmod, not recover-the-file).

## Acceptance Criteria

### Agent
- [x] `exec` is added to the axis-B verb anchor, and the scan carries the VERB alongside the
      path — resolution alone cannot decide executability, so the verb must survive the
      extraction rather than being discarded by the second grep as it is today.
- [x] Executability is required **only** for `exec`-position references. A `bash` / `sh` /
      `.` / `source` / `python3` reference to a mode-644 file is correct and must NOT fire —
      that precision is the reason to key on the verb instead of testing `-x` on everything.
- [x] `resolves-but-not-executable` is reported as its own class, distinct from DANGLING,
      with its own count and its own remediation line. Collapsing it into DANGLING would
      tell an operator to recover a file that is present.
- [x] The check FIRES on the real tree as it stands today, naming `lib/build.sh` — this is
      ground truth extracted from the live defect, not a synthetic mutant.
- [x] `--json` carries the new class separately (`not_executable_count` + `not_executable[]`)
      and the exit code accounts for it, so a scripted caller cannot read green over it.
- [x] Both output paths state the scope: the check covers `exec`-POSITION references only,
      and a bare `"$FRAMEWORK_ROOT/bin/foo"` invoked with no verb also needs the bit and is
      NOT covered. A green must not read as "every framework invocation resolves" (T-2680).
- [x] Fixtures extend `tests/framework-dangling-ref-fixtures.sh` and pin, at minimum: the
      firing case, the `bash`-on-644 false-positive guard, and a mutant — removing the `-x`
      test must redden the firing assertion and nothing else.
- [x] The full fixture suite passes and the check still runs clean as a guard-layer member
      apart from the one genuine finding it now reports.
- [x] **(added mid-build — see Decisions)** The finding is acknowledged in a git-tracked
      ledger rather than left permanently red, and the ledger is itself pinned by fixtures:
      an acknowledged entry must still be NAMED on the clean path, removing the entry must
      re-fire, a commented-out path must NOT acknowledge, and an ABSENT ledger must
      acknowledge nothing rather than excusing everything.

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

# The suite, including the mutant and the ledger's fail-open guard.
bash tests/framework-dangling-ref-fixtures.sh > /tmp/.t3145-fix 2>&1 && grep -q "20 passed, 0 failed" /tmp/.t3145-fix

# exec is in the anchor, in the COMMITTED script (T-3086: assert against HEAD, not the worktree).
git show HEAD:scripts/check-framework-tracking-drift.sh > /tmp/.t3145-head 2>&1 && grep -q "python3|python|exec" /tmp/.t3145-head

# The -x test is keyed on the verb, not applied blanket.
grep -q 'verb" = "exec"' /tmp/.t3145-head

# The real tree: exec references are now examined (59 -> 62) and the ledger is honoured.
bash scripts/check-framework-tracking-drift.sh --json > /tmp/.t3145-json 2>&1
python3 -c "import json,sys; d=json.load(open('/tmp/.t3145-json')); assert d['refs_checked']==62, d['refs_checked']; assert d['not_executable_acknowledged_count']==1, d; assert d['not_executable_count']==0, d; assert d['ok'] is True; print('real-tree envelope ok')"

# The acknowledged finding is still NAMED on the clean path — a green must not hide it.
bash scripts/check-framework-tracking-drift.sh > /tmp/.t3145-text 2>&1 && grep -q "ACK  .FRAMEWORK_ROOT/lib/build.sh" /tmp/.t3145-text

# The ledger is tracked (T-2681) — an untracked ledger makes the guard non-reproducible.
git ls-files --error-unmatch .context/checks/framework-notexec-allowlist

## RCA

**Symptom:** `fw build` — advertised at `fw help` line 85 — exits 126 Permission denied,
while `check-framework-tracking-drift.sh` reported the tree clean with 59 references resolved.

**Root cause:** axis B's verb anchor listed `.` · `source` · `bash` · `sh` · `python3` ·
`python` and omitted `exec`. All three exec-position references in the vendored tree were
therefore outside the candidate set — never examined, not merely passing. The omitted verb is
the only one where file mode matters, because `exec` does not fall back to an interpreter.

**Why structurally allowed:** three independent guards each had a blind spot that the others
did not cover, and their intersection was empty. Axis A asks "is the file tracked?" — it is.
A mode-drift comparison asks "does the index disagree with the disk?" — it does not: both say
644, agreeing on the wrong value. Axis B asks "does the reference resolve?" — but only for
verbs it was told about. Each guard was correct within its own frame; nothing owned the
question "can the thing this code execs actually be exec'd?". The peer report that surfaced
it (`framework:pickup` 149, 832-Workflow-designer) described a DIFFERENT root cause — `fw
upgrade` stripping exec bits, index 100755 vs disk not-executable — which our tree is clean
of. Ours was worse-hidden: the wrong value was committed, so there was nothing to drift from.

**Prevention:** distinct from the fix. The `-x` test is pinned by fixture 9 against the live
defect and by a MUTANT (fixture 14) that strips the test and asserts the check then reports
the dead reference as clean — so a future simplification that removes it fails the suite
rather than silently restoring the blind spot. Fixture 10 is the opposite guard: `bash` /
`source` / `python3` on a mode-644 file must NOT fire, which is what stops the obvious
"simplify it, just test -x everywhere" change from landing. The upstream fix is filed at
`framework:pickup` offset 162; the local detector is what survives a re-vendor that does not
carry it.

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

### 2026-09-25 — leave the finding red, or acknowledge it?

- **Chose:** acknowledge `lib/build.sh` in a git-tracked ledger, counted and reported on the
  clean path, with a stated removal condition.
- **Why:** the check is a `# guard-layer: source` member and `release.yml` gates both build
  jobs on the guard layer, so a hard-red check blocks every build. The defect is vendored and
  G-062 forbids patching it here, so "fix it" is not available to us — the alternative to
  acknowledging is a check that fires forever on something we are structurally barred from
  fixing, which is the alarm-fatigue failure T-2818 documented and the reason nobody reads a
  permanently-red gate. The acknowledgement is not a mute button: the entry is printed on the
  clean path, so a green still says "one acknowledged", never "nothing to see".
- **Rejected:** (a) leaving it red — teaches the operator that this check's red is normal,
  which destroys the signal the task exists to create; (b) `chmod +x` locally — erased by the
  next re-vendor, and it would make the check green while the upstream defect persists, so
  the next consumer to vendor it inherits a dead verb with our detector saying clean;
  (c) exempting `lib/build.sh` in the code — an exemption with no ledger and no removal
  condition is indistinguishable from never having looked.
- **Note:** this AC did not exist when the task was scoped. It was added, not quietly
  satisfied — the ledger is a real mechanism with its own failure modes, so it got its own
  five fixtures (15-19) including the fail-open guard.

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

### 2026-09-25T14:08:56Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3145-axis-b-omits-exec--the-one-invocation-fo.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-37754b66
- **Timestamp:** 2026-09-25T14:16:15Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-09-25T14:16:12Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
