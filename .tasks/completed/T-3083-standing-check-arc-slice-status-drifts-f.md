---
id: T-3083
name: "Standing check: arc slice status drifts from its task register with nothing
  detecting it"
description: >
  An arc slice register records status per slice (built/partial/unbuilt) and binds
  each slice to a task. The status is hand-maintained, so a slice goes stale the moment
  its task completes and nobody returns to it. Caught by hand three times in two days
  on arc-011: S6 read unbuilt after T-3070 shipped, S11 read 'Crontab written, NOT
  installed' a full day after it was installed and verified firing, and S7/S10 read
  unbuilt after the rail was proven live. The drift UNDER-claims, which invites work
  to be done twice. check-arc-claim-drift.sh cannot see it: it judges CLOSED arcs
  for prover bindings and says so in its own scope line. T-3077 added a cross-check
  but it runs only at that task's completion — a one-shot, not a standing guard.

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
created: 2026-09-23T09:46:25Z
last_update: 2026-09-23T09:51:31Z
date_finished: 2026-09-23T09:51:31Z
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
  - ts: '2026-09-23T09:47:27Z'
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

# T-3083: Standing check: arc slice status drifts from its task register with nothing detecting it

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] **It fires on the drift that actually happened.** Pointed at arc-011 as it stood
      at `3f0962566` — extracted from git, not synthesised — it names **S6, S7, S8 and
      S10**. Against the current tree it is clean.
      (AMENDED mid-build: this originally claimed all THREE manual catches including
      S11. It does not catch S11, and that is not a bug. At that commit S11 read
      `unbuilt` while T-3068 was still ACTIVE, so status and location agreed. S11's
      drift was a stale NOTE — "Crontab written, NOT installed" after it was installed —
      which is note-vs-reality and undecidable mechanically. Corrected rather than
      ticked as written, and the limitation is now stated in the check's own SCOPE.)
- [x] **Scope is stated in every output path** (T-2680): it detects a slice status
      STALE against its task's location. It does NOT judge whether a note is accurate,
      whether `partial` is the right call, or whether a slice should exist — a green
      must not read as "the register is correct"
- [x] **It covers the arcs the existing guard cannot see.** `check-arc-claim-drift.sh`
      judges CLOSED arcs for prover bindings; this judges IN-PROGRESS arcs for slice
      staleness. The boundary is stated so the two are not mistaken for each other
- [x] **Fail-closed:** an unparseable arc file, an arc with zero slices, or absent
      python3 exits 2 — never a clean pass. "I could not look" and "I looked and found
      nothing" must not share an exit code
- [x] An unresolvable `task:` reference fires too — a slice bound to a task that exists
      nowhere is un-auditable by construction
- [x] Allowlist at `.context/checks/arc-slice-drift-allowlist` (git-tracked per T-2681),
      entries counted and reported but non-firing, each needing a cited reason
- [x] Hermetic fixtures with a seam for the arcs dir and tasks dir (PL-213), covering:
      the real drift, a clean tree, an unresolvable task, zero-slices, and an
      allowlisted entry. Mutation-proven — disabling the staleness arm reddens a named
      fixture
- [x] Carries the `# guard-layer: source` marker so `run-guard-layer.sh` picks it up,
      and the runner still reports it as a member

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

bash -n scripts/check-arc-slice-drift.sh

# 15 assertions. D1 runs against arc-011 as it stood at 3f0962566, extracted from git
# rather than synthesised, and must name the slices that were genuinely stale. D6 pins
# what this CANNOT see (a stale note) so the limit is a claim, not a later surprise.
bash tests/arc-slice-drift-fixtures.sh

# The current register is clean by the check's own reckoning.
bash scripts/check-arc-slice-drift.sh --quiet

# Structural pins: the guard-layer marker (without it the check never runs
# automatically, which is the dormant-tooling class PL-168), and the scope constant.
head -3 scripts/check-arc-slice-drift.sh > /tmp/.t3083h 2>&1 && grep -q 'guard-layer: source' /tmp/.t3083h
grep -q 'SCOPE = ' scripts/check-arc-slice-drift.sh

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

### 2026-09-23 — the guard I deferred three times, and the catch it would have missed

- **What changed:** this was deferred three times as "the real G-019 close", each time
  with the note that T-3077's cross-check "runs only at that task's completion — a
  one-shot, not a standing guard". In between I caught the same drift by hand three
  times in two days. The recurrence rate, not the idea, is what finally justified it:
  a defect I personally re-discover every ~16 hours is not folklore, it is an unguarded
  mechanism.
- **The load-bearing fixture is extracted from git, not written from memory.** D1 runs
  the check against arc-011 exactly as it stood at `3f0962566` — the tree in which I
  caught S6 by hand — and it names S6, S7, S8 and S10. A fixture built from what I
  BELIEVE the drift looked like would encode the same assumption that let the drift
  survive in the first place.
- **It does NOT catch one of the three, and I amended the AC rather than the claim.**
  AC1 originally said it fires on all three manual catches including S11. It does not.
  At that commit S11 read `unbuilt` while T-3068 was still ACTIVE, so status and
  location AGREED — the falsehood was in the NOTE ("Crontab written, NOT installed",
  a full day after it was installed and verified firing). Note-vs-reality is
  undecidable mechanically: nothing in the repo knows what `/etc/cron.d` contains.
  So the limit is now stated in the check's own SCOPE line, printed on every output
  path, and pinned by fixture D6 — a passing assertion that this case passes, so the
  gap is a documented claim rather than something discovered later by someone trusting
  a green.
- **Why `partial` is not judged either:** a slice can legitimately read `partial` with
  its task completed (arc-011 S5 does — the right event on the wrong mechanism). Firing
  on that would make the check permanently red on a correct register, which is the
  T-2818 corrosion shape. Only `unbuilt`-with-a-completed-task is unambiguous.
- **Cost vs estimate:** BVP 57, cost unmeasured — the estimator still refuses an
  un-started task on the cost axis, so the quadrant was inferred from T-3082, its value
  twin, which measured hv-lc. Actual scope matched: one check, one fixture suite.
- **Triggered:** nothing new. The check is a guard-layer member (verified via
  `run-guard-layer.sh --list`), so it runs in CI on every push without further wiring.

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

### 2026-09-23T09:46:25Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3083-standing-check-arc-slice-status-drifts-f.md
- **Context:** Initial task creation

### 2026-09-23T09:47:50Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-728cc755
- **Timestamp:** 2026-09-23T09:51:35Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-09-23T09:51:31Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Standing slice-drift check shipped: fires on the real historical drift extracted from git (S6/S7/S8/S10), clean on the current tree, 15 fixtures, both arms mutation-proven, discovered by the guard-layer runner. Its one blind spot (stale notes) is stated in its own scope and pinned by fixture D6.
