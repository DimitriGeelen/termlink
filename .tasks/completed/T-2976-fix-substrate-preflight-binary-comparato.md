---
id: T-2976
name: "Fix substrate-preflight binary comparator (tag/mtime-based)"
description: >
  S-2/C-12 check half: preflight Check 4 comparator misreads binary freshness. Rework
  comparison to be tag/mtime-based. Evidence: consolidated C-12; run2 contradiction
  4.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [value-review, arc:arc-009]
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
created: 2026-09-19T21:58:50Z
last_update: 2026-09-24T20:48:26Z
date_finished: 2026-09-24T20:48:26Z
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

# T-2976: Fix substrate-preflight binary comparator (tag/mtime-based)

## Context

Filed from value-review C-12 (2026-09-19, citing run2 F9 / run4 A4 / run5 F-02+E-49):
"the preflight comparator can't distinguish '174 versions behind' from '1 product
commit behind'" and proposes "compare against last tagged release or build-vs-source
mtime, not the governance-inflated commit counter." **Investigation (this round)
found the proposed fix already landed as T-2226 (commit `622173df4`, 2026-06-14 —
over 3 months before the review ran).** `crates_unchanged_since_binary()` in
`scripts/substrate-preflight.sh:526` does exactly the ask: within a release line it
walks `git log HEAD~delta..HEAD -- crates/` to distinguish a real feature-relevant
gap from a governance-inflated patch-counter drift, and fails safe to WARN (never
silently PASS) on any uncertainty — cross-minor/major boundary, shallow clone,
unparseable version. `PL-220` (learnings.yaml, sourced from T-2226) documents this
exact fix. Run2's own F9 framing ("structurally unsatisfiable") reads as a
misdiagnosis made without checking current code; run4/run5's independent reading —
the check is *correctly* detecting real staleness, and the 74-75 day firing streak
is an *unactioned ops gap* (S-3/T-2977: reinstall + restart), not a comparator bug —
is what this session's direct code read + live run confirms. See Verification.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `crates_unchanged_since_binary()` exists in `scripts/substrate-preflight.sh`
      and implements a feature-relevant (crates/-diff-based), not raw-counter-based,
      staleness comparison, with fail-safe-to-WARN on any uncertainty.
- [x] The fix predates the value-review finding that requested it (T-2226,
      2026-06-14, commit `622173df4`) — the C-12/F9/S-2 ask is already satisfied
      in the current tree; no further comparator code change is required.
- [x] A live run of `substrate-preflight.sh` on this host confirms the comparator
      is exercising the intended logic (not dead code) and reports a real,
      explained staleness reason rather than an inflated-counter false positive.


## Verification

grep -n "crates_unchanged_since_binary" scripts/substrate-preflight.sh > /tmp/.t2976a.out && grep -q "crates_unchanged_since_binary()" /tmp/.t2976a.out
git log --format=%H -1 622173df4 > /tmp/.t2976b.out 2>&1 && grep -q "622173df4" /tmp/.t2976b.out
bash scripts/substrate-preflight.sh --no-heartbeat > /tmp/.t2976c.out 2>&1; grep -qE "T-2226|older than project VERSION" /tmp/.t2976c.out

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

**Symptom:** Value-review finding C-12/F9 (2026-09-19) asserted the preflight binary
comparator was "structurally unsatisfiable" (can't tell 174-versions-behind from
1-commit-behind) and proposed a tag/mtime-aware, feature-relevant comparison as the fix.

**Root cause:** Not a defect in `scripts/substrate-preflight.sh` — the defect was in the
review's own diagnosis. `crates_unchanged_since_binary()` (T-2226, commit `622173df4`,
2026-06-14) already implements exactly the proposed fix and has been live for over 3
months before the review ran. The review inferred "comparator broken" purely from the
canary log's long unbroken firing streak, without reading current source to check
whether that streak reflected a false positive (comparator bug) or a true positive
(genuine unaddressed staleness, i.e. the S-3/T-2977 ops half). It was the latter — run4
and run5's independent readings of the same evidence ("the tools work, nobody acted on
them") were correct; run2's F9 framing was not.

**Why structurally allowed:** The value-review methodology has no step requiring a
REPAIR-class finding to grep/read current source for an existing fix before filing new
remediation work — it can file a task against a defect that a prior task already closed,
and nothing catches the duplication until someone works the new task and checks.

**Prevention:** This task's own `## Verification` block is now that check, permanently
recorded (asserts the fix function exists, cites the commit, and confirms the live
script exercises the fixed logic). No new lint/gate is added — the general fix (review
process re-checking source before filing REPAIR items) is a methodology change for the
human running future value-review passes, not something this task can enforce in code.

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

### 2026-09-24 — already-fixed, closing via re-verification
- **What changed:** Filed as a code-fix task; investigation found the code fix (T-2226)
  already landed 2026-06-14, over 3 months before this task was filed from the
  2026-09-19 value review. No code change was needed — only re-verification.
- **Plan impact:** S-2 (this task, C-12 check-half) closes as already-satisfied. S-3
  (T-2977, C-12 ops-half: reinstall binary, restart hub) remains the genuine open gap
  and is unaffected by this closure.
- **Triggered:** No new sub-task. Cross-referenced in T-2977 is not required — its own
  filing already stands on its own evidence (E-49, three installed binaries).

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

### 2026-09-19T21:58:50Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2976-fix-substrate-preflight-binary-comparato.md
- **Context:** Initial task creation

### 2026-09-19T22:08:33Z — status-update [task-update-agent]
- **Change:** tags: +arc:arc-009

### 2026-09-24T20:46:53Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-b3e50c23
- **Timestamp:** 2026-09-24T20:48:27Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-09-24T20:48:26Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
