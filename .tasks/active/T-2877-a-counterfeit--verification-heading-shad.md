---
id: T-2877
name: "A counterfeit '## Verification' heading shadows the real one and the gate runs
  prose"
description: >
  extract_verification_block takes the FIRST ^## Verification match. T-2873 carried
  an orphaned template block (a stray --> at line 111 with no opening <!-- above it)
  whose wrapped fragment left '## Verification` instead of a Human AC here...' at
  column 0, 25 lines above the genuine heading. The gate therefore extracted template
  prose ('1. Open https://example.com/dashboard in browser') and the task's five real
  verification commands were unreachable. check-verification-misfile.sh (T-2831) is
  structurally blind: it looks for commands in the wrong SECTION, and here the section
  itself is counterfeit. Corpus: 23 of 2596 files carry two ## Verification headings;
  22 extract correctly, so measured incidence of actual mis-extraction is 1/2596.

status: started-work
workflow_type: build
owner: claude-code
horizon: now
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
created: 2026-09-01T21:19:14Z
last_update: 2026-09-01T21:20:41Z
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
  - ts: '2026-09-01T21:20:42Z'
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

# T-2877: A counterfeit '## Verification' heading shadows the real one and the gate runs prose

## Context

`extract_verification_block` (`.agentic-framework/lib/verification-port.sh:175`) is
`sed -n '/^## Verification/,/^## /p' | sed '$d' | tail -n +2 | comment_strip | grep -v ...`.
It takes the **first** `^## Verification` match, so any line at column 0 that merely looks
like that heading shadows the genuine one and the gate runs whatever follows the
counterfeit.

Found while verifying T-2873, which carried an orphaned template block: a stray `-->` at
line 111 with **no opening `<!--` anywhere above it**, and a wrapped fragment leaving
`` ## Verification` instead of a Human AC here... `` at column 0, twenty-five lines above
the real heading. `.tasks/templates/default.md` has that same sentence correctly indented
at line 59, so the template is not the source — an edit re-wrapped it.

**Measured, not inferred.** Running the real extractor over the corpus:

| predicate | fires on | of which true positives |
|---|---|---|
| `>1` occurrence of `^## Verification` | 23 / 2596 | 1 |
| an orphaned `-->` with no opener | 14 / 2596 | 1 |
| **extracted block contains markdown prose** | **1 / 2596** | **1** |

The first two are why this check does not gate on the obvious signal: a guard that is
wrong 22 times out of 23 teaches its operator to stop reading it, which is the fatigue
mechanism T-2818 documented from the other direction. They are reported as *diagnosis on
an already-firing file* instead, because that is exactly what an operator needs in order
to fix it.

**Both failure directions are silent.** The extracted prose either evals as bash (the
T-2990 / T-2991 hazard — that is how 56 MB of ImageMagick PostScript once reached this
repo's root) or is refused as unparseable, reporting a cause that has nothing to do with
the actual fault. And an empty extraction returns 0 immediately
(`update-task.sh:1184` — `[ -z "$verify_cmds" ] && return 0`), so "no commands to run"
and "all commands passed" remain the same output.

**Two instances existed; both were repaired, not acknowledged.** T-2873 (active) and
T-274 (completed, whose single real command `grep -q "Auto-fix" ...` had been stranded
under the counterfeit heading while its genuine `## Verification` sat empty). The command
was preserved and moved into the real section; it passes.

**One adjacent finding, deliberately not made into a task.** `sed '$d'` unconditionally
drops the block's last line, assuming it is the next heading — so a task whose
`## Verification` is the *final* section loses its last command silently. Corpus scan:
1 file (T-110), whose dropped line is a bookkeeping `**Change:** status:` entry, not a
command. Real but currently harmless; recorded here rather than inflated into a task.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] **The defect is reproduced against the REAL extractor, not a re-typed copy.** The check must call the framework's own `extract_verification_block` (`.agentic-framework/lib/verification-port.sh`) rather than reimplement the "find `## Verification`" rule. Two copies of that rule drift, and the copy that drifts is the one that quietly stops catching things (the T-2818 argument, and `update-task.sh`'s own T-2921 comment makes the same point about its former inline copy).
- [x] **The firing predicate is chosen by measurement, and the rejected candidates are recorded with their measured miss/over-fire counts.** Two obvious predicates were measured and are NOT adequate on their own: `>1` occurrence of `^## Verification` fires on 23/2596 files of which 22 extract correctly, and an orphaned `-->` fires on 14 files of which 1 mis-extracts. A guard that is wrong 22 times out of 23 is the fatigue shape T-2818 documented. Record the numbers, not just the conclusion.
- [x] **`scripts/check-verification-heading-shadow.sh` exists and asserts the property that matters** — that what the P-011 gate will actually execute is command-shaped, not markdown prose. Exit 0 clean / 1 firing / 2 tooling, **fail-closed**: a missing tasks dir, an unsourceable extractor, absent `python3`, or a corpus of zero task files all exit 2, never a vacuous clean.
- [x] **It is load-bearing against the real instance, extracted from git rather than a synthetic mutant.** Pointed at `T-2873` as it stood at commit `93cb3d8d9` (orphaned `-->` at line 111, counterfeit heading at line 88), it must FIRE; against the repaired tree it must be clean.
- [x] **Carries `# guard-layer: source` and is picked up by the runner.** `bash scripts/run-guard-layer.sh --list` names it, and a full run does not report it as unclassified.
- [x] **Fixture suite with mutants pinned.** `tests/verification-heading-shadow-fixtures.sh`, weighted to the firing cases and the false-positive guards, with at least one mutant per detector arm that turns the suite red when the arm is disabled. A green fixture suite that cannot go red is not evidence.
- [x] **Acknowledgements are git-tracked under `.context/checks/` (T-2681)**, one signature per line with a cited reason, counted and reported but non-firing. Empty on purpose if the one real instance was repaired rather than acknowledged.
- [x] **Scope is stated on every output path (T-2680).** The check detects a counterfeit/shadowed heading; it does NOT verify that a task's verification is adequate, that its commands test its ACs, or that a task has any verification at all — a task with an empty `## Verification` passes this check and still gates on nothing.


## Verification

bash scripts/check-verification-heading-shadow.sh --quiet
grep -q "guard-layer: source" scripts/check-verification-heading-shadow.sh
bash tests/verification-heading-shadow-fixtures.sh > /tmp/.t2877.txt 2>&1 && grep -q "0 failed" /tmp/.t2877.txt
bash scripts/run-guard-layer.sh --list > /tmp/.t2877b.txt 2>&1 && grep -q "check-verification-heading-shadow.sh" /tmp/.t2877b.txt
mkdir -p /tmp/.t2877lb/active && git show 93cb3d8d9:.tasks/active/T-2873-termlinkremoteinject-sends-bare-string-k.md > /tmp/.t2877lb/active/T-2873.md && ! bash scripts/check-verification-heading-shadow.sh --tasks-dir /tmp/.t2877lb --quiet

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

### 2026-09-01T21:19:14Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2877-a-counterfeit--verification-heading-shad.md
- **Context:** Initial task creation

### 2026-09-01T21:20:41Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
