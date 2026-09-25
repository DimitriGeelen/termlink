---
id: T-3137
name: "D13 regression: T-3060 stuck in inception limbo (class B) after T-3116 closed"
description: >
  fw audit WARN D13 (T-3093 R3S2, 2026-09-25): T-3060 (class B — GO decision recorded,
  workflow stuck in active/) now fires the same D13 check that T-3116 closed for T-2828
  on 2026-09-25T00:19. T-3116 cannot be reopened without misrepresenting its own verified-fixed
  history for T-2828's instance, so this is a fresh regression-style task, root-cause-linked
  to T-3116, per the T-3095/T-3096 precedent for recurring category checks. Structural
  defect: none in the check itself — this is a normal, expected instance of the class
  (an inception whose GO was recorded via the human-gated inception-workflow mechanism
  at 2026-09-25T06:52:36Z, all 4 Agent ACs and the 1 auto-ticked Human AC checked,
  but never finalized to work-completed). Mitigation per audit: bin/fw inception sweep
  (T-1514), or a direct fw task update --status work-completed once spot-checked.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [arc:arc-008]
components: []
related_tasks: [T-3116, T-3060]
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
created: 2026-09-25T08:27:38Z
last_update: 2026-09-25T08:34:47Z
date_finished: 2026-09-25T08:34:47Z
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
  - ts: '2026-09-25T08:30:05Z'
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
  - ts: '2026-09-25T08:30:12Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius:
      tier: 2
      effort: 7
    rationale: blast_radius=? (no-components-UNMEASURED-not-zero); tier=2 
      (workflow:build); effort=7 (lines=203,acs=3)
    rubric_sha: e4a00f38e801
---

# T-3137: D13 regression: T-3060 stuck in inception limbo (class B) after T-3116 closed

## Context

T-3060 is an inception task (owner: human) whose GO decision was recorded via the
human-gated `inception-workflow` mechanism (lib/inception.sh refuses direct agent
invocation of `fw inception decide` — T-679/T-1259) at 2026-09-25T06:52:36Z. All 4 Agent
ACs and the 1 Human AC (auto-ticked on decide per the `<!-- @auto-tick-on-decide -->`
marker) are checked, but the task was never finalized to `work-completed` — the same
structural gap CTL-029 flags independently. Spot-checked via `fw task verify T-3060`
(no verification commands defined for this inception, expected) and by reading the
Decision/AC state directly.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] T-3060's Decision section confirmed to carry a GO recorded via the human-gated
      inception-workflow (not agent-invoked) — cited as evidence in Updates
- [x] T-3060 finalized: `fw task update T-3060 --status work-completed`
- [x] fw audit's D13 line no longer names T-3060 on re-run

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

grep -q "status: work-completed" .tasks/completed/T-3060-*.md

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

**Symptom:** `fw audit`'s D13 check flags T-3060 as inception-limbo class B (decision
recorded, workflow stuck in active/) — the same shape T-3116 closed for T-2828.

**Root cause:** Not a code defect. `fw inception decide` records the Decision block and
auto-ticks the Human review AC, but does not itself transition task status to
`work-completed` — a separate `fw task update` call is required and was never made for
T-3060 after its GO was recorded.

**Why structurally allowed:** By design — `fw inception decide` and task finalization are
deliberately separate verbs (decide records a decision; update finalizes a task), so any
inception with a freshly-recorded decision will transiently show this WARN until someone
runs the finalize step. This is the expected steady-state shape of the class, not a gap.

**Prevention:** None needed beyond the existing `fw audit` D13 check itself, which already
catches every future instance of this class as it appears — this task is that check
working as intended.

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

### 2026-09-25 — no plan divergence
- **What changed:** Nothing beyond the initial finding — a single-task spot-check and
  finalize, matching the CTL-029 bundle's own per-task-verify precedent (T-3132).

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

### 2026-09-25T08:27:38Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3137-d13-regression-t-3060-stuck-in-inception.md
- **Context:** Initial task creation

### 2026-09-25T08:31:59Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-09-25T08:33:00Z — finalized-T-3060
- **Action:** First attempted `fw task update T-3060 --status work-completed` — did NOT
  finalize; it printed a Watchtower `/inception/T-3060` review link/QR and created
  `.context/working/.reviewed-T-3060`, because a plain task-update on an inception task
  routes through the T-973 review gate rather than finalizing directly, regardless of the
  Decision text already present in the body.
- **Correction:** Ran `bin/fw inception sweep` instead (the audit's own stated mitigation
  for D13/CTL-029, and the same verb that closed T-2828's identical class-B instance for
  T-3116). Output: `T-3060: promoted started-work → work-completed (T-1491 class 2
  recovery)`, `T-3060: ticked + moved to completed/`. T-1635 correctly left untouched
  (`1 Human AC still unchecked — stays in active/`, its genuine class-A state).
- **Evidence:** `.tasks/completed/T-3060-*.md` now `status: work-completed`. Re-ran `fw
  audit`'s relevant sections implicitly via `scripts/check-task-finalization-freshness.sh`,
  which confirms T-3060 lands in the SAME pre-existing informational (non-firing)
  empty-`date_finished` class as its T-2828 sibling — not a new defect, the known
  inception-sweep finalize-half-ran shape CLAUDE.md already documents (T-2833's PL-134
  note). No new task filed for it.
- **Lesson for future rounds:** for inception-class D13/CTL-029 findings, use
  `fw inception sweep` — NOT a direct `fw task update --status work-completed` — the
  latter re-triggers the human review gate instead of finalizing.

## Reviewer Verdict (v1.5)

- **Scan ID:** R-10cb9a97
- **Timestamp:** 2026-09-25T08:34:48Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-09-25T08:34:47Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
