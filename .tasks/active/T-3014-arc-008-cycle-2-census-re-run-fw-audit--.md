---
id: T-3014
name: "arc-008 cycle-2 census: re-run fw audit + fw doctor, diff vs cycle-1, file
  unfiled findings"
description: >
  arc-008 cycle-2 census: re-run fw audit + fw doctor, diff vs cycle-1, file unfiled
  findings

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components:
  - .context/audits/arc-008-cycle2-census.md
  - .context/audits/arc-008-cycle1-census.md
related_tasks: [T-2938, T-2939, T-2940]
arc_id: arc-008
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
created: 2026-09-20T10:14:56Z
last_update: '2026-09-20T10:17:36Z'
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
  - ts: '2026-09-20T10:17:08Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 4
      D3: 3
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=4 (body:fw-audit-or-doctor); D3=3
      (body:component-discoverability); D4=2 (body:env-class-handled); 
      F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: e4a00f38e801
cost_estimate_proposed:
  - ts: '2026-09-20T10:17:24Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius:
      tier: 2
      effort: 8
    rationale: blast_radius=? (no-components-UNMEASURED-not-zero); tier=2 
      (workflow:build); effort=8 (lines=228,acs=5)
    rubric_sha: e4a00f38e801
  - ts: '2026-09-20T10:17:36Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 2
      effort: 8
    rationale: blast_radius=3 (2-components); tier=2 (workflow:build); effort=8 
      (lines=228,acs=5)
    rubric_sha: e4a00f38e801
---

# T-3014: arc-008 cycle-2 census: re-run fw audit + fw doctor, diff vs cycle-1, file unfiled findings

## Context

arc-008's rule is that verification of a finding is **the re-run of the audit in the next
cycle, not self-assertion**. Cycle 1 (2026-09-09, `.context/audits/arc-008-cycle1-census.md`)
captured the complete finding census: `fw audit` 368 pass / 77 warn / 5 fail, `fw doctor`
14 warn / 0 fail. Eleven days later the three Q1 tasks filed off that census (T-2938 cron
drift, T-2939 uninstalled substrate-smoke canary, T-2940 D2 review queue) are all parked to
human review with GO verdicts, but **nobody has re-run the census**, so which cycle-1
findings actually cleared is unmeasured — and the arc rule says that re-run is the proof.

This task is the cycle-2 census: re-run both verbs, write the durable cycle-2 record, diff
it against cycle-1 (cleared / persisting / new), and file any FAIL that has no task, per the
arc rule that every individual FAIL and WARNING becomes its own governed task and findings
sharing a root cause are linked, never merged.

Scope boundary: this task **measures and files**. It does not remediate any finding — each
remediation is its own task, which is the arc's whole premise.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `.context/audits/arc-008-cycle2-census.md` exists and records, for both `fw audit`
      (all sections) and `fw doctor`: the summary counts, the capture timestamp/commit, and
      every raw FAIL line verbatim
- [x] The census carries an explicit cycle-1 → cycle-2 diff naming each cycle-1 FAIL as
      CLEARED / PERSISTS / SUPERSEDED, with the evidence for each verdict
- [x] Every FAIL in the cycle-2 audit is either mapped to an existing task ID or filed as a
      new task, and the census records the mapping (no FAIL left unattributed)

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

test -f .context/audits/arc-008-cycle2-census.md
grep -q "^## Cycle-1 to cycle-2 diff" .context/audits/arc-008-cycle2-census.md
grep -q "^## FAIL to task mapping" .context/audits/arc-008-cycle2-census.md
grep -q "^## Audit FAIL" .context/audits/arc-008-cycle2-census.md

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

## Evolution

### 2026-09-20 — the re-run contradicted two closed tasks
- **What changed:** The census was filed expecting to confirm remediation. It did the
  opposite for D8/D8b: T-2941 and T-2942 were closed `work-completed` on 2026-09-09 and
  eleven days later both audit lines are byte-identical (5 [TODO], 10/10). The distinction
  that emerged is instance-vs-mechanism — both tasks filled the handover that existed that
  day; neither touched `handover.sh` or the PreCompact auto-commit that mints a new
  violation on every compaction. `fw doctor` emitted HANDOVER STALE during this session,
  unprompted, which is the mechanism firing in real time.
- **Plan impact:** "diff the two cycles" was scoped as bookkeeping. It is the arc's actual
  verification instrument, and this is the first cycle where it caught something that
  self-assertion had already marked done.
- **Triggered:** T-3015 (the mechanism, linked to T-2941/2942/2943 rather than reopening
  either — arc rule: findings sharing a root cause are linked, never merged).

### 2026-09-20 — two framework surfaces disagree about the same 30 tasks
- **What changed:** Not in the plan at all. CTL-029 flags T-2938/T-2939/T-2940 as
  "completable, not closed"; attempting that close returned `Sovereignty gate (R-033):
  owner is human`. Found by hitting the refusal, not by reading the warning list — I had
  gone to close those three believing a prior session had simply forgotten to.
- **Plan impact:** The premise that arc-008's Q1 tasks were "unfinished" was wrong; their
  parked-to-review state is terminal for an agent. 30 of 94 warnings are an un-actionable
  class.
- **Triggered:** PL-376 (the terminal-state rule), T-3016 (the contradiction itself).

### 2026-09-20 — the census nearly lost its own evidence
- **What changed:** The full audit takes ~8 minutes. I misread a slow run as a dead one
  (`pgrep 'fw audit'` misses it — the process is named `audit.sh`), and the retry's `rm`
  deleted the live run's output file from under it. The run survived on a deleted inode and
  the output was recovered through `/proc/<pid>/fd/1`; the second invocation was correctly
  refused rc=75 by the flock.
- **Plan impact:** None to the deliverable — the data is complete. Worth recording because
  the lock behaved exactly as designed and the near-loss was entirely mine.
- **Triggered:** nothing filed; no framework defect involved.

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

### 2026-09-20T10:14:56Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3014-arc-008-cycle-2-census-re-run-fw-audit--.md
- **Context:** Initial task creation
