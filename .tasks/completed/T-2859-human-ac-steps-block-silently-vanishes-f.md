---
id: T-2859
name: "Human AC Steps block silently vanishes from the approval page when the heading is not exactly Steps"
description: >
  The Watchtower review renderer matches the Human AC Steps heading with an exact startswith('**Steps:**'). A parenthesized variant such as '**Steps (copy-paste):**' drops the entire Steps block — including the operator's copy-pasteable command — while Expected and If-not still render, so the page looks complete. Normalize the 2 deviating active tasks and file the silent-drop defect upstream (renderer is vendored, G-062).

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [scripts/check-human-ac-steps-heading.sh, tests/human-ac-steps-heading-fixtures.sh]
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
created: 2026-08-30T10:27:52Z
last_update: 2026-08-30T10:42:16Z
date_finished: 2026-08-30T10:42:16Z
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

# T-2859: Human AC Steps block silently vanishes from the approval page when the heading is not exactly Steps

## Context

The rendered review page is the approval surface: it is what a human reads before
stamping an action. Its Human-AC block has three fields — Steps, Expected, If not —
and the renderer parses only two of them symmetrically.

`.agentic-framework/web/blueprints/tasks.py` (~line 419) captures the remainder of the
`**Expected:**` and `**If not:**` marker lines into the field's content. The
`**Steps:**` branch discards it (`current_content = []; continue`). Combined with an
exact `startswith('**Steps:**')` test, that yields two silent losses:

- **class 1** — a heading like `**Steps (copy-paste):**` never matches, so the entire
  Steps block is dropped;
- **class 2** — a canonical heading with content on the same line loses that content,
  and where the whole list was on that line the page renders **no Steps section at all**.

Measured across `.tasks/active/`: **8 of 129** Steps headings affected — 2 class 1,
6 class 2. The page returns 200 and still shows Expected and If-not, so nothing looks
wrong. The worst instance, T-2522, is a human *decision* task whose three options were
absent from the page asking for the decision. One affected task (T-1696) had already
been reported to the operator as a verified, stamp-ready approval.

The renderer is vendored, so the fix belongs upstream (G-062); what is landed here is
the local normalisation of all 8 files plus the guard that survives a re-vendor.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] Every Human AC `Steps` heading across `.tasks/active/` uses the exact canonical form `**Steps:**` (the only form the renderer matches)
- [x] The Steps block, including its copy-pasteable command, renders on the review page for both previously-affected tasks (T-1696, T-2858)
- [x] The renderer's silent-drop behaviour is filed upstream — it is vendored (`.agentic-framework/web/blueprints/tasks.py:419`), so it is NOT patched locally (G-062)
- [x] RCA names why a deviating heading costs the whole block silently rather than degrading visibly
- [x] A guard detects the next deviating heading, so this cannot silently recur

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

bash scripts/check-human-ac-steps-heading.sh > /tmp/.t2859-guard.out 2>&1 && grep -q "clean" /tmp/.t2859-guard.out
bash tests/human-ac-steps-heading-fixtures.sh > /tmp/.t2859-fix.out 2>&1 && grep -q "0 failed" /tmp/.t2859-fix.out
! grep -rn '^[[:space:]]*[*][*]Steps (' .tasks/active/
! grep -rEn '^[[:space:]]*[*][*]Steps:[*][*][[:space:]]+[^[:space:]]' .tasks/active/
grep -q '^# guard-layer: source' scripts/check-human-ac-steps-heading.sh
curl -sf "$(cat .context/working/watchtower.url)/review/T-2858" -o /tmp/.t2859-p1.out && grep -q 'check-installed-binary-drift.sh' /tmp/.t2859-p1.out
curl -sf "$(cat .context/working/watchtower.url)/review/T-2522" -o /tmp/.t2859-p2.out && grep -q 'Confirm no production topic relies on content-time' /tmp/.t2859-p2.out

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

**Symptom:** Human-AC `Steps` blocks — including the operator's copy-pasteable command —
were missing from rendered approval pages, while `Expected:` and `If not:` rendered
normally, so the page looked complete. Worst instance: T-2522, a human *decision* task
whose three choices were entirely absent from the page the operator was asked to decide on.

**Root cause:** `.agentic-framework/web/blueprints/tasks.py` (~line 419) parses the three
Human-AC fields asymmetrically. `**Expected:**` and `**If not:**` each capture the
remainder of their marker line (`rest = stripped[len(marker):]`) into `current_content`.
The `**Steps:**` branch sets `current_content = []` and `continue`s, discarding it. Two
distinct losses follow: a heading that is not *exactly* `**Steps:**` never matches
`startswith`, so the whole block is dropped (class 1); a canonical heading with content on
the same line has that content discarded (class 2), rendering no Steps section at all when
the entire list was on that line.

**Why structurally allowed:** the drop is silent and *partial*. Nothing errors, the page
returns 200, and the two sibling fields still render — so the failure presents as a
complete page rather than a broken one. No guard compared the task file's Steps content
against what the page actually rendered, and the review pipeline's own gates check that a
Human AC *has* Steps/Expected/If-not in the **source**, never that they survive rendering.
The consequence lands precisely at the sovereignty boundary: the human approves an action
whose command the page never showed them — Directive #2 (no silent failures) failing where
it is most expensive.

**Contributing judgment error:** a prior session's check flagged T-1696's heading and the
finding was dismissed as a false alarm on the reasoning that the parenthesized form "reads
fine". It was a true positive; the page had been carrying no Steps for that task, and the
link was reported to the operator as verified and stamp-ready. Dismissing a guard's finding
on how the *source* reads, without looking at what the *consumer* produced, is the same
mistake in miniature.

**Prevention:** `scripts/check-human-ac-steps-heading.sh` (guard-layer member) fires on both
classes and is pinned by `tests/human-ac-steps-heading-fixtures.sh` (15 assertions, weighted
to the firing cases and fail-closed paths). The renderer itself is vendored, so it is **not**
patched here (G-062) — filed upstream at `framework:pickup` offset 74 with the three-line
symmetric fix. The guard is the half that survives a re-vendor.

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

### 2026-08-30T10:27:52Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2859-human-ac-steps-block-silently-vanishes-f.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-786836ec
- **Timestamp:** 2026-08-30T10:42:19Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Per-AC findings:**

- **AC#3 (Agent)** — The renderer's silent-drop behaviour is filed upstream — it is vendored (`.agentic-framework/web/blueprints/tasks.py:419`), so it is NOT patched locally (G-062)
  - **AC-verify-mismatch** (narrow, heuristic) — `path=agentic-framework/web/blueprints/tasks.py in: The renderer's silent-drop behaviour is filed upstream — it is vendored (`.agentic-framework/web/blueprints/tasks.py:419`), so it is NOT patched local`

### 2026-08-30T10:42:16Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
