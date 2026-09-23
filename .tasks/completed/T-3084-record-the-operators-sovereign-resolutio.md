---
id: T-3084
name: "Record the operator's sovereign resolutions for arc-011 (SQ-1,3,4,6,7)"
description: >
  Five sovereign questions were answered by the operator: SQ-1 the injector belongs to TermLink; SQ-3 yes to termlink artifact put/get CLI verbs; SQ-4 urgent must never inject into a BUSY prompt, it shortens the wait rather than bypassing the check; SQ-6 backfill the delivered watermark once, recorded explicitly as a backfill rather than silently; SQ-7 T-3068's sixth AC is obsolete because the truthful L3 now comes from the proven injector path. Record each in the arc register with the decision and its consequence, and unpark the tasks each was blocking, so the decisions survive this session.

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
created: 2026-09-23T11:54:09Z
last_update: 2026-09-23T11:58:04Z
date_finished: 2026-09-23T11:58:04Z
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

# T-3084: Record the operator's sovereign resolutions for arc-011 (SQ-1,3,4,6,7)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] All five answered questions (SQ-1, SQ-3, SQ-4, SQ-6, SQ-7) carry a `status:` and
      a `resolution:` in `.context/arcs/arc-011.yaml`, each recording the DECISION and
      what it consequentially authorises or forbids — not merely "answered"
- [x] The record attributes each decision to the operator and does not present my
      recommendation as the reasoning. I proposed; the operator decided
- [x] SQ-7's resolution is reflected where it bites: T-3068's sixth AC is marked
      obsolete with the reason, rather than ticked as though it had been met
- [x] No arc-011 sovereign question remains open, and the arc YAML parses with all 12
      slices intact and every `task:` still resolving
- [x] `bash scripts/check-arc-slice-drift.sh` stays clean after the edits — the guard
      shipped one unit ago must not be reddened by this one

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

# The register parses, keeps all 12 slices, and carries no open sovereign question.
python3 -c "import yaml; d=yaml.safe_load(open('.context/arcs/arc-011.yaml')); assert len(d['slices'])==12; assert not [x for x in d['sovereign_questions'] if 'status' not in x]"

# Every answered question records a DECISION and its CONSEQUENCE, not merely 'answered'.
# Scoped to the FIVE this task recorded. SQ-2 and SQ-5 were resolved in earlier runs
# under a different convention; asserting over them would be claiming this task did
# something it did not.
python3 -c "import yaml,sys; M={'SQ-1','SQ-3','SQ-4','SQ-6','SQ-7'}; q=yaml.safe_load(open('.context/arcs/arc-011.yaml'))['sovereign_questions']; bad=[x['id'] for x in q if x['id'] in M and ('Operator decision' not in x.get('resolution','') or 'CONSEQUENCE' not in x.get('resolution',''))]; print('incomplete:', bad or 'none'); sys.exit(1 if bad else 0)"

# SQ-7 applied where it bites, and marked obsolete rather than silently dropped.
grep -q 'OBSOLETE — resolved by the operator' .tasks/active/T-3068-give-the-wake-consumer-a-trigger-supervi.md

# The guard shipped one unit ago is not reddened by this one.
bash scripts/check-arc-slice-drift.sh --quiet

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

### 2026-09-23 — I corrupted the register I was recording decisions into, and caught it before shipping

- **What changed:** the task looked like transcription. It was not. My first write put the
  decisions into unquoted YAML scalars containing `": "` — `status: RESOLVED ... operator:
  the injector belongs to TERMLINK` — which is invalid YAML, and it broke
  `.context/arcs/arc-011.yaml` outright. That is the decisions.yaml corruption class this
  repo has hit four times, committed by me, in the act of recording five sovereign
  decisions into the register.
- **What saved it was verifying before writing, not after.** The first attempt wrote the
  file and then parsed it, so the corruption reached disk and I recovered with
  `git checkout`. The second attempt parses the STRING and asserts the slice count and
  the absence of open questions BEFORE `write_text` is ever called. A register that
  cannot be parsed cannot be repaired by the thing that broke it, so the parse belongs
  before the write. Worth generalising: any edit to a structural register should validate
  in memory first.
- **The verification over-reached and was narrowed rather than satisfied.** My check
  asserted that every resolved question carries `Operator decision` and `CONSEQUENCE`.
  SQ-2 and SQ-5 were resolved in earlier runs under a different convention and failed it.
  The honest fix was to scope the assertion to the five THIS task recorded — asserting
  over SQ-2/SQ-5 would have been claiming this task did something it did not, and
  retrofitting them would have meant rewriting the record of earlier decisions to suit a
  later test.
- **On authorship:** every resolution says "Operator decision" because that is what it is.
  I proposed five recommendations and the operator accepted them; the register records the
  decision and its consequence, not my reasoning. SQ-7 in particular is applied where it
  bites — T-3068's sixth AC is marked OBSOLETE with the reason and the original text kept
  verbatim beneath it, rather than ticked as though met or silently deleted.
- **Cost vs estimate:** unscored on the cost axis (the estimator still refuses un-started
  tasks, third run reporting this). Priced mentally as transcription; actual cost was
  dominated by the corruption and recovery.
- **Triggered:** SQ-6 needs its own task — the backfill must be written as a backfill,
  distinguishable from a watermark the injector earned. Not opened here: context is at the
  stop condition.

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

### 2026-09-23T11:54:09Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3084-record-the-operators-sovereign-resolutio.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-92db7eb9
- **Timestamp:** 2026-09-23T11:58:06Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-09-23T11:58:04Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Five operator decisions recorded in the arc register with decision and consequence; SQ-7 applied to T-3068's sixth AC as obsolete with reason; no open sovereign questions remain; slice-drift guard still clean.
