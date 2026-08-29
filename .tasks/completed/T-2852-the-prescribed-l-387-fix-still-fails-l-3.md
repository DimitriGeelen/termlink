---
id: T-2852
name: "The prescribed L-387 fix still fails L-387 above 64KiB"
description: >
  The repo-wide safe rewrite for L-387 (out=$(cmd); echo "$out" | grep -q PAT) returns
  141 when the captured output exceeds the pipe capacity and the match is early. The
  vendored detector exempts that exact shape, so it cannot catch its own recommendation.

status: work-completed
workflow_type: build
owner: claude-code
horizon: null
tags: []
components: [scripts/check-task-template-idioms.sh, tests/l387-boundary-fixtures.sh, tests/task-template-idioms-fixtures.sh]
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
created: 2026-08-29T10:24:58Z
last_update: 2026-08-29T10:51:24Z
date_finished: 2026-08-29T10:51:24Z
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
  - ts: '2026-08-29T10:26:15Z'
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

# T-2852: The prescribed L-387 fix still fails L-387 above 64KiB

## Context

T-2743 already measured that the prescribed L-387 rewrite is itself an L-387
failure above the pipe capacity, and corrected `.tasks/templates/default.md`.
This task is not that discovery — it is the reconciliation. The same claim
exists in four copies with no transclusion (the T-2484 class), and only the
template was fixed.

**Left alone deliberately.** The idiom appears in ~661 historical task files
under `.tasks/`. Those are not rewritten: they are a record of what was
believed at the time, most are already completed, and mass-editing them would
churn the corpus without changing any future outcome. This task changes
PRESCRIBED guidance — the template and CLAUDE.md — which is what new tasks copy
from. Two of the four copies are vendored (G-062) and were filed upstream at
`framework:pickup` offset 70 rather than patched.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] The failure is reproduced and its boundary measured, not asserted: `echo "$out" | grep -q PAT` under `set -o pipefail` returns 141 when `$out` exceeds the pipe capacity AND the match is early enough for grep to exit before echo finishes writing. A fixture pins both the failing case and the passing herestring so the claim can be falsified by anyone
- [x] `.tasks/templates/default.md` no longer PRESCRIBES the failing shape. The template is the highest-leverage surface here because every future task copies its Verification block from it, so a wrong idiom there reproduces itself indefinitely
- [x] The CLAUDE.md T-2818 section no longer calls the echo-pipe form "SIGPIPE-immune". The claim is replaced with one that states the actual bound, because a qualified-but-true rule is safer than a simple-but-false one
- [x] The vendored detector's unconditional echo/printf exemption is filed upstream, NOT patched locally (G-062). The filing states the DIRECTION of the failure — the detector cannot flag the shape it recommends, so the blind spot is self-sealing
- [x] Every verification command written for this task uses the herestring form, so the task is its own smallest proof
- [x] Where the fix is a judgement call rather than a defect, it is left alone and said so — this task changes prescribed guidance, not every historical occurrence in 2500 completed task files

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
       `bin/fw reviewer T-XXX 2>&1 | grep -q "Overall:.*PASS"` added to ## Verification.
-->

## RCA

**Symptom:** The rewrite the framework prescribes to AVOID L-387 is itself an
L-387 failure. `out=$(cmd 2>&1); echo "$out" | grep -q PAT` under `set -o
pipefail` returns 141 once the capture exceeds the pipe capacity and the match
is early. Measured: clean at 32 KiB, 141 at 128 KiB, nondeterministic at 65536
exactly. Independently measured by T-2743 on a real 146,366-byte page.

**Root cause:** The claim "echo upstream is SIGPIPE-immune" misidentifies what
bounds the write. The buffer is the PIPE's (65536 bytes on Linux), not echo's.
`echo` writes the whole capture regardless of size, so it blocks on a full pipe
exactly like any other producer, takes SIGPIPE when `grep -q` exits on the first
match, and pipefail propagates 141. The form is safe only when the capture fits
the pipe, or when the match is late enough that grep consumes the whole stream —
which is why it passes in the common case and looked correct for months.

**Why structurally allowed:** Two compounding gaps.

1. The claim exists as FOUR copies with no transclusion — the task template,
   CLAUDE.md, `policy/anti-patterns.yaml`, and `static_scan.py` — and nothing
   verified they agreed. T-2743 corrected exactly one. This is the T-2484 class
   (the charter sentence as three copies) applied to verification guidance, and
   CLAUDE.md was the worst copy to leave stale because it is auto-loaded into
   every session: every agent read "SIGPIPE-immune" at start while the template
   said the opposite.

2. The detector is self-sealing. `detect_l387_sigpipe_risk` exempts an
   `echo`/`printf` upstream UNCONDITIONALLY, so it can never flag the shape it
   recommends. Any corpus scan it runs reports that shape as clean by
   construction, which reads as evidence the recommendation is sound. A detector
   that structurally cannot falsify its own advice will keep confirming it —
   the same shape as T-2680, where a guard's green was read as broader than what
   it had checked.

The failure DIRECTION is what makes it urgent rather than untidy: the gate
BLOCKS a task whose verification actually passed. Per T-2818, a gate that blocks
incorrectly teaches its operator that failures are noise and that `--force` is
the normal way past them — and a verification gate people routinely force is a
verification gate that no longer verifies. It also fails rarely and
nondeterministically, hardest when a command emits abnormally large output, i.e.
exactly when something has already gone wrong.

**Prevention:**
- `tests/l387-boundary-fixtures.sh` (9 assertions) pins the measurement itself,
  so the claim is falsifiable by anyone on any host rather than resting on
  whoever measured it last.
- `check-task-template-idioms.sh` guards the template against re-acquiring the
  shape, and now carries the tracked allowlist its five siblings have, so a
  bounded prescription can be acknowledged with a cited reason instead of being
  deleted or ignored. Fixtures 21 → 33, including a leg proving that removing an
  entry re-fires the site.
- CLAUDE.md now states the bound, leads with the file-redirect default, and
  NAMES the two vendored copies that still disagree — so the next reader is told
  the disagreement exists rather than discovering it.
- Both vendored copies filed upstream at `framework:pickup` offset 70 (G-062).

## Verification

# Every line below uses the herestring form on purpose — this task is its own
# smallest proof that the corrected idiom works.
out=$(bash tests/task-template-idioms-fixtures.sh 2>&1 || true); grep -q "33 passed, 0 failed" <<< "$out"
out=$(bash tests/l387-boundary-fixtures.sh 2>&1 || true); grep -q "9 passed, 0 failed" <<< "$out"
out=$(bash scripts/check-task-template-idioms.sh --no-heartbeat 2>&1 || true); grep -q "4 acknowledged" <<< "$out"
out=$(grep -c "SIGPIPE-immune" CLAUDE.md || true); grep -qx "0" <<< "$out"
out=$(git ls-files .context/checks/task-template-idioms-allowlist || true); grep -q "task-template-idioms-allowlist" <<< "$out"

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
# Why not `cmd | grep -q PAT` (L-387): P-011 runs each line under `set -eo
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

### 2026-08-29T10:24:58Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2852-the-prescribed-l-387-fix-still-fails-l-3.md
- **Context:** Initial task creation

### 2026-08-29T10:26:15Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-308fa8e0
- **Timestamp:** 2026-08-29T10:51:29Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-29T10:51:24Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
