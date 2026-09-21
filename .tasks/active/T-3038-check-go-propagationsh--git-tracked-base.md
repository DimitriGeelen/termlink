---
id: T-3038
name: "check-go-propagation.sh + git-tracked baseline ledger (fires on NEW leaks only)"
description: >
  Local detection for the GO-propagation leak T-3003 measured: 80/167 GO inceptions
  strictly unlinked, 13 new in 2026-08. Ships a check plus a git-tracked baseline
  ledger so the 80 existing instances are frozen and visible while only NEW leaks
  fire — the T-2818 fatigue lesson (a guard that is permanently red is a guard nobody
  reads).

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [arc:arc-009, go-propagation, guard]
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
created: 2026-09-21T11:09:14Z
last_update: 2026-09-21T11:23:12Z
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
  - ts: '2026-09-21T11:12:23Z'
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

# T-3038: check-go-propagation.sh + git-tracked baseline ledger (fires on NEW leaks only)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `scripts/check-go-propagation.sh` exists and carries the `# guard-layer: source` marker so `run-guard-layer.sh` picks it up
- [x] Exit contract: 0 = no unacknowledged leak, 1 = a NEW unlinked GO inception, 2 = tooling — **fail-closed**: absent `python3`, an unreadable ledger, or a corpus with zero inception files all exit 2, never a vacuous clean
- [x] Git-tracked baseline ledger at `.context/checks/go-propagation-allowlist`
      <!-- Filename deviates from this AC as first written (`-baseline`). Renamed to
           `-allowlist` to match the ten sibling ledgers under .context/checks/
           (charter-drift, alloc-sink, drain-sink, silent-exit, busy-spin,
           verification-misfile, stranded-finalized, mcp-parity-census,
           error-code-emission, platform-lock, version-derivation). Substance of the
           AC is unchanged and met: git-tracked, under .context/checks/, one
           `<task-id>  # <reason>` per line, counted-and-reported, non-firing.
           Recorded rather than silently amended — T-3038. -->
- [x] A GO inception with empty `related_tasks` that is NOT in the baseline **fires**; deleting a baseline line re-fires that inception — the load-bearing property, demonstrated both directions
- [x] Output on BOTH paths (clean and firing) states the census and a scope disclaimer (T-2680): it detects GO inceptions with no forward link, it does NOT audit whether the follow-on work is adequate or whether the GO was right
- [x] The loose-vs-strict split T-3003 measured is preserved: strictly-unlinked fires, loosely-linked (a back-reference exists elsewhere) is reported non-firing — so the 4 genuine orphans stay distinguishable from the 76 metadata gaps
- [x] Fixtures at `tests/go-propagation-check-fixtures.sh` pin the exit codes, the baseline suppression, the re-fire on removal, and at least one false-positive guard (a GO inception that IS properly linked must never fire)

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

# ── T-3038 verification (L-387-safe: redirect to file, never `cmd | grep -q`) ──
bash tests/go-propagation-check-fixtures.sh > /tmp/.t3038-fix.out 2>&1 && grep -q "0 failed" /tmp/.t3038-fix.out
bash scripts/check-go-propagation.sh > /tmp/.t3038-real.out 2>&1 && grep -q "0 firing" /tmp/.t3038-real.out
bash scripts/check-go-propagation.sh > /tmp/.t3038-scope.out 2>&1 && grep -q "does NOT audit" /tmp/.t3038-scope.out
bash scripts/run-guard-layer.sh --list > /tmp/.t3038-gl.out 2>&1 && grep -q "check-go-propagation.sh" /tmp/.t3038-gl.out
bash scripts/run-guard-layer.sh --list > /tmp/.t3038-gl2.out 2>&1 && grep -q "go-propagation-check-fixtures.sh" /tmp/.t3038-gl2.out
git ls-files --error-unmatch .context/checks/go-propagation-allowlist
git ls-files --error-unmatch tests/go-propagation-check-fixtures.sh
test -x tests/go-propagation-check-fixtures.sh

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

### 2026-09-21 — the loose "orphan" axis cannot distinguish work from bookkeeping

- **What changed:** T-3003 named four genuine orphans (T-954, T-955, T-958, T-1698)
  using a loose predicate: strictly unlinked AND no other task mentions the ID at all.
  Filing T-3040 to triage those four — a task whose description lists all four IDs —
  made every one of them read as "mentioned". The orphan count fell from 4 to 3 to 0
  with no remediation work done on any of them. The axis measures whether an ID has
  been written down somewhere, not whether anything was done about it.
- **Plan impact:** the orphan flag stays, but it is an annotation and must never become
  a firing gate. It is reported alongside the strict predicate rather than replacing it.
  The strict predicate (related_tasks on either side) is the one that fires.
- **Triggered:** no new task; recorded here and in the check's own output wording so the
  next reader does not mistake a falling orphan count for progress.

### 2026-09-21 — two defects found by RUNNING the check, not by reading it

- **What changed:** (1) frontmatter timestamps are emitted sometimes quoted and sometimes
  bare. An unstripped quote made `fromisoformat` raise, age read as None, and the record
  reached the firing branch by FALLING THROUGH rather than by being old — a task decided
  today fired. T-2828 fired correctly only by luck. (2) The verdict scanner had to be
  ordered so NO-GO and DEFER can never read as GO; a substring match instead of a prefix
  match fires on inceptions that were correctly declined and have no follow-on by design.
- **Plan impact:** both are now pinned as fixture cases 7 and 8 and demonstrated with
  mutants, because neither is visible by inspection — only by execution against a corpus
  that contains the shape.
- **Triggered:** the fixture suite's weighting toward regression cases over happy paths.

### 2026-09-21 — the GO detector is deliberately conservative and does NOT reconcile with T-3003

- **What changed:** the check counts 164 GO-recorded inceptions where T-3003 measured 167.
  The 80 strictly-unlinked figure agrees exactly (71 firing + 9 in grace), so the gap is in
  GO DETECTION, not in the link predicate. The direction is safe — a false negative, three
  inceptions the verdict scanner does not recognise as GO — but it is unexplained.
- **Plan impact:** the ledger was baselined at 71 rather than 80 because 9 were inside the
  grace window at baseline time; those fire around 2026-09-28 if still unlinked. The census
  is reported on every run so the discrepancy stays visible instead of being absorbed.
- **Triggered:** nothing filed. Reconciling 164 vs 167 is a follow-on if the gap matters;
  recorded here so a future reader does not assume the two measurements agree.

### 2026-09-21 — shipped the T-2830 defect while building a guard against it

- **What changed:** the `## Verification` block for this task was first inserted before
  `## Decision` and therefore landed at the end of `## Decisions` — commands under a
  neighbouring heading, exactly the misfile class T-2830/T-2831 documented. P-011 would
  have found an empty Verification section and passed VACUOUSLY on a task whose whole
  claim is that a guard is load-bearing.
- **Plan impact:** none to the deliverable; caught before commit by reading the section
  order rather than trusting the insert, then confirmed clean by
  `scripts/check-verification-misfile.sh` (2747 files, 0 misfiled).
- **Triggered:** no new task — the guard already exists and worked. Recorded because the
  lesson is that having the guard did not stop the mistake; running it did.

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

### 2026-09-21T11:09:14Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3038-check-go-propagationsh--git-tracked-base.md
- **Context:** Initial task creation

### 2026-09-21T11:12:45Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
