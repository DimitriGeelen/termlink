---
id: T-2982
name: "File upstream: hook-telemetry flock fix (L-023)"
description: >
  S-9/C-20: hook-telemetry counter writes race without flock; vendored (G-062) — file
  upstream with the fix, register in .vendor-divergence.yaml. Evidence: consolidated
  C-20.

status: started-work
workflow_type: build
owner: agent
horizon: now
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
created: 2026-09-19T22:05:16Z
last_update: 2026-09-20T22:12:32Z
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
  - ts: '2026-09-20T08:45:10Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 2
      D3: 3
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=2 
      (body:telemetry-or-audit-entry); D3=3 (body:component-discoverability); 
      D4=2 (body:env-class-handled); F-RECALL=0 (no-signal); F-ORCH=0 
      (no-signal)
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

# T-2982: File upstream: hook-telemetry flock fix (L-023)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] The lost-update race in `_fw_telemetry_increment` is REPRODUCED by a concurrency
      harness, with measured expected-vs-actual counts recorded in this task (an
      asserted race is not evidence of one).
- [x] The corruption window is characterised: whether concurrent readers can observe a
      truncated/empty counter file, not merely a count one low.
- [x] A candidate fix is written and shown to eliminate the loss under the SAME harness
      (0 lost increments across repeated runs).
- [x] The fix's per-call overhead is measured against the 5ms T-1626 budget that the
      function's own comment cites as the reason it avoids a subprocess.
- [x] Defect + reproduction + measured fix are FILED at `framework:pickup` (the code is
      vendored — G-062 forbids patching it here), and the filing offset is recorded.
- [x] The divergence is registered in `.vendor-divergence.yaml` with `status:
      filed-upstream` and a cited reason.

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

# The harness must exist, and must still DEMONSTRATE the race against the
# unfixed vendored function (a harness that cannot go red proves nothing).
test -x tests/hook-telemetry-race-fixtures.sh
bash tests/hook-telemetry-race-fixtures.sh > /tmp/.t2982 2>&1 && grep -q "ALL ASSERTIONS PASSED" /tmp/.t2982
# The divergence register must parse and must carry this task's entry.
python3 -c "import yaml,sys; d=yaml.safe_load(open('.vendor-divergence.yaml')); sys.exit(0)"
grep -q "T-2982" .vendor-divergence.yaml

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

**Symptom:** Hook fire/failure counters under `.context/working/` are wrong and
visibly malformed — a bare fragment line `6`, and `error-watchdog` present twice
(56 and 9). Measured under load, 477-479 of 480 increments are lost.

**Root cause:** `lib/hook-telemetry.sh::_fw_telemetry_increment` performs an
unlocked read-modify-write: `mapfile` the whole file, modify in memory, then
`printf ... > "$file"`. Two writers read the same base and the last wins. The
`>` truncation makes it far worse than a lost update — a concurrent writer's
`mapfile` observes a near-empty file, bases its increment on ~0 and writes back
~1, so the counter collapses toward 1 rather than drifting down.

**Why structurally allowed:** the function deliberately avoids a subprocess.
Its own comment states the reason: "Pure bash — no subprocess fork — to keep
per-fire overhead under the T-1626 5ms budget." That trade was never measured.
Measured here, flock costs 2.877ms against a 5.000ms budget — the constraint
that justified the unsafe design does not bind.

The second-order reason it survived: the corruption biases the guard layer
toward SILENCE. Duplicate keys inflate the summing reader's denominator, so a
repeatedly-failing hook reads as a lower failure ratio and stays under
threshold. A defect that makes alarms quieter does not announce itself.

**Prevention:** detection already exists and is good —
`scripts/check-hook-counter-integrity.sh` fires daily, names this exact
mechanism, and flags reader disagreement. What was missing is the *fix*, and
the fix cannot live here: the code is vendored, so a local patch is erased by
the next re-vendor (G-062). Prevention is therefore (a) the upstream filing at
`framework:pickup` offset 132 carrying a reproduction and a measured fix, and
(b) `tests/hook-telemetry-race-fixtures.sh`, whose leg 1 asserts the defect
still reproduces — so a future re-vendor that lands the fix turns leg 1 red and
tells us, rather than leaving us guessing.

## Evolution

### 2026-09-21 — the fix was never the hard part; the justification was

- **What changed:** The task was filed as "file upstream the flock fix", which
  reads as a clerical errand. Two things turned out differently. First, the
  severity was understated: this is not a counter that drifts a few percent
  low, it is a counter that collapses to ~1 (480 expected, 1-3 observed). The
  `>` truncation, not the lost update, is the mechanism. Second, the real
  obstacle to fixing it upstream was never "write a lock" — it was the
  function's own comment asserting a 5ms budget as grounds for staying
  lock-free. An upstream filing that ignored that objection would have been
  declined on sight, so the filing had to *measure* it: 0.306ms unfixed,
  2.877ms with flock, budget 5.000ms.
- **Plan impact:** The deliverable shifted from "report a race" to "refute the
  stated design constraint with numbers". The harness exists to carry that
  evidence upstream, not merely to prove a race locally.
- **Also found:** the local detection was already complete and already firing —
  `check-hook-counter-integrity.sh` had been naming this mechanism daily, and
  its log was 7KB and non-empty. Worth recording that the gap was never
  detection; it was that detection had nowhere to route a fix, because the code
  is vendored.
- **Triggered:** none. Deliberately no local patch (G-062). The re-verification
  step is recorded in `.vendor-divergence.yaml` under `reverify:` rather than
  as a new task, because it is a step in the existing pre-re-vendor checklist.

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

### 2026-09-19T22:05:16Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2982-file-upstream-hook-telemetry-flock-fix-l.md
- **Context:** Initial task creation

### 2026-09-19T22:08:36Z — status-update [task-update-agent]
- **Change:** tags: +arc:arc-009

### 2026-09-20T22:12:32Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)
