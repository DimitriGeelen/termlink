---
id: T-2891
name: "T-2889's verification block landed under ## RCA, so P-011 passed vacuously"
description: >
  The 7-line verification block written for T-2889 was inserted after the ## Verification section's trailing comment block but BEFORE the ## RCA heading was accounted for, so it landed inside ## RCA. P-011 extracts from ## Verification only, found it empty, and passed vacuously — the identical defect T-2830 committed and T-2831 shipped check-verification-misfile.sh to detect. The 7 commands were all executed by hand and returned rc=0, so T-2889's claims are true; what is false is any implication that a gate proved them. Repair the placement, confirm the misfile checker actually fires on this shape (if it does not, that is the more valuable finding), and re-run the gate for real.

status: work-completed
workflow_type: build
owner: agent
horizon: null
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
created: 2026-09-03T12:12:26Z
last_update: 2026-09-03T12:15:41Z
date_finished: 2026-09-03T12:15:41Z
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

# T-2891: T-2889's verification block landed under ## RCA, so P-011 passed vacuously

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] **The block is moved into `## Verification` and its placement is asserted
      structurally, not by eye.** An `awk` walk that tracks the most recent `## ` heading
      reports the verification lines under `## Verification` — the same walk that exposed
      the defect. My original check spanned `## Verification` through `## Decisions` and
      therefore could not see a block sitting in `## RCA` between them; a scoping bug in
      the check is what let the misfile through, so the replacement check must be the
      narrow one.
- [x] **Whether `check-verification-misfile.sh` fires on this shape is MEASURED and
      recorded either way.** T-2831 shipped it to detect exactly this defect. If it fires,
      the gap was that nothing ran it at completion time. If it does NOT fire, that is the
      more valuable finding — the checker looks for command lines in a *wrong* section, and
      a block in `## RCA` of a file whose `## Verification` is empty is its central case.
      A checker that misses its own central case is a guard reporting green over the defect
      it was built for, and must be filed rather than quietly worked around.
- [x] **T-2889 is re-completed with the gate genuinely executing its 7 commands**, not
      `--force`d. Forcing a completion to make a gate look like it ran is the same
      dishonesty inverted (T-2831). If the gate cannot be re-run because `work-completed`
      is terminal, that limitation is recorded plainly rather than papered over.
- [x] **T-2889's Recommendation/RCA text is not left claiming the gate proved anything it
      did not.** The task's own record states that its commands were executed by hand and
      that the first completion passed vacuously.

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

# --- T-2891 verification ---
# AC1: the block sits under ## Verification and NOWHERE else. Narrow walk (nearest
# preceding '## ' heading) — the wide span is the bug that let the misfile through.
awk '/^## /{h=$0} /^grep -qx .enrichment_status/{print h}' .tasks/completed/T-2889-enrich-s-2026-0903-1102-clear-the-d8-aud.md | sort -u > /tmp/.t2891-place.txt
grep -qx '## Verification' /tmp/.t2891-place.txt
test "$(wc -l < /tmp/.t2891-place.txt)" = "1"
# AC2: the guard that detects this shape scans the corpus clean after the repair
bash scripts/check-verification-misfile.sh --quiet
# AC4: T-2889's own record states the vacuous pass and the measured guard behaviour
grep -q 'passed P-011 vacuously' .tasks/completed/T-2889-enrich-s-2026-0903-1102-clear-the-d8-aud.md
grep -q 'DOES fire on this exact shape' .tasks/completed/T-2889-enrich-s-2026-0903-1102-clear-the-d8-aud.md
# T-2889's substance still holds after the move (the enrichment itself is unaffected)
bash scripts/check-handover-staleness.sh --json > /tmp/.t2891-hs.json 2>/tmp/.t2891-hs.err && grep -q '"unenriched_in_window": 0' /tmp/.t2891-hs.json


## RCA

**Symptom:** T-2889 completed reporting "Acceptance criteria: 6/6 checked" having executed
**zero** verification commands. Its 7-line block was written, was correct, and passed when
run by hand — but the P-011 gate never saw it. No error, no warning: an empty
`## Verification` section and a passing run are the same output.

**Root cause:** the block was inserted by anchoring on the last comment line of the
`## Verification` template, then walking forward past every remaining comment and blank
line to find the insertion point. That walk ran straight through the section boundary and
landed inside `## RCA`, the next heading. The anchor was a *comment*, which is not a
section delimiter; nothing in the insertion logic knew where `## Verification` ended.

**Why structurally allowed — two independent failures, and the second is the interesting
one.** (1) Nothing runs `check-verification-misfile.sh` at completion time. It was measured
against the misfiled file and **fires correctly**, naming all 7 lines and attributing them
to `## RCA`; detection was never the gap. This is the "shipped but nothing executes it"
class T-2683 found across the whole static-check layer and T-2686 closed for CI but not for
the completion path. (2) **My own placement self-check was mis-scoped and reported success.**
It set `p=1` at `## Verification` and cleared at `## Decisions`, spanning `## RCA` in
between — so it printed the block and looked like confirmation. A check whose span is wider
than the property it asserts will confirm whatever you already believe; that is the same
defect as the thing it was checking for, one level up, and it is why the misfile survived a
deliberate verification step.

**Prevention:** the placement assertion is now the *narrow* walk — track the nearest
preceding `## ` heading and require the set of headings above the block to be exactly
`{## Verification}` (`sort -u` + a 1-line count), so a block in any other section fails
rather than passes. That assertion is itself in T-2891's `## Verification` block, so P-011
runs it. `check-verification-misfile.sh` scans the corpus clean (2611 files, 0 misfiled).
The residual gap — nothing runs the guard layer at completion time, only in CI — is real,
is not closed here, and is the reason this could recur on the next task; it belongs to the
T-2686 execution-coverage arc rather than to this repair.

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

### 2026-09-03T12:12:26Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2891-t-2889s-verification-block-landed-under-.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-4207adf0
- **Timestamp:** 2026-09-03T12:15:43Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-09-03T12:15:41Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
