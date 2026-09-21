---
id: T-3037
name: "File upstream: rail_project_label() guesses an identity instead of refusing (G-062)"
description: >
  rail_project_label() falls back to basename ${PROJECT_ROOT:-$PWD} when the label is absent, so 11 framework:pickup filings were attributed 'root' instead of '010-termlink' purely by invocation cwd, defeating the T-2816 self-filter. An absent label is honestly unknown and filters as unknown; 'root' is confidently wrong and filters as somebody else. Vendored under .agentic-framework/ so it is filed upstream, never patched locally.

status: captured
workflow_type: build
owner: agent
horizon: next
tags: [arc:arc-009, upstream, identity]
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
created: 2026-09-21T10:48:39Z
last_update: 2026-09-21T10:48:39Z
date_finished: null
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

# T-3037: File upstream: rail_project_label() guesses an identity instead of refusing (G-062)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

`rail_project_label()` (vendored, `.agentic-framework/`) falls back to
`basename ${PROJECT_ROOT:-$PWD}` when the label is absent. Measured cost, recorded at
`.context/project/learnings.yaml:4418` and promoted as **PL-373** (from T-2955):
**11 of our own `framework:pickup` filings were attributed `root` instead of `010-termlink`**,
purely by invocation cwd — defeating the T-2816 self-filter, the mechanism that keeps our own
filings from firing the G-063 canary at us.

**The learning is already promoted; the fix was never filed.** PL-373 states the principle
("a fallback that GUESSES an identity is worse than one that refuses") and the guessing
fallback is still in the vendored code. That gap is what this task closes.

**The defect is the guess, not the path.** An absent label is honestly unknown and filters
as unknown; `root` is confidently wrong and filters as somebody else. T-2905 centralised the
label so it could not be spelled three ways, but left a derived fallback underneath —
reintroducing the property the centralisation removed.

**Why this is not "adopt AEF's path model instead" (settled with AEF, 2026-09-21).**
AEF rules that a project identity IS the absolute root directory path, `serialize()` in
`lib/aef_address.py` refuses anything else, and `elide_path()` is display-only. That model
is correct for what it does, and the distinction that resolves it is:

| | needs | path suitable? |
|---|---|---|
| **Addressing** (reach/start a project) | uniqueness within a host; a real dir to `cd` to | **yes** — adopt AEF's model as-is |
| **Attribution** (whose filing is this?) | stability when the code relocates | **no** — a git worktree is a different absolute path |

A worktree checkout of this project has a different absolute path, so under a path-derived
attribution our two checkouts attribute as two projects and the self-filter — which matches a
constant — stops recognising one of them. That is the eleven-filings bug in a new costume.
So: two fields, not one string doing both jobs. AEF's path is **bound** (declared per project),
not **inferred at call time**; `basename $PWD` is the inference. The rule is "declared, not
inferred" — it was never "not paths".

Adjacent, NOT in this task's scope (file separately if pursued): the Watchtower route is
derived from the path, so the same project in a worktree renders two routes.

**G-062 — the code is vendored.** A local patch is erased by the next re-vendor (8 wholesale
vendor events in 124 commits). This task files upstream; it does not patch
`.agentic-framework/`.

### Why this task is PARKED at `captured` (finding, 2026-09-21)

Authoring this file proved the "scored before started" binding **unsatisfiable through the
sanctioned path**, in a stronger form than previously recorded. Three refusals, in order:

1. `P-002` refuses a Bash write to the task body: *"Task T-3037 has status 'captured'"*.
2. `G-020` refuses `fw bvp estimate` on placeholder ACs — so the ACs must exist first.
3. `P-002` then refuses **`fw bvp estimate` itself**: *"Task T-3037 has status 'captured'"*.

(3) is the decisive one, and it is **not an allowlist gap**. The gate's own message invites
filing one if the command "genuinely only reads" — it does not: the estimator writes
`bvp_scores_proposed` into the frontmatter. The gate is correct to block it. Therefore the
scorer is reachable only from `started-work`, and a task cannot be scored before it is
started. The binding and the gate cannot both hold.

Resolving that is a Sovereign question (open, priority 5), so this task is left `captured`
with real ACs and no score rather than started to force the scorer. Do not "fix" this by
starting the task — that silently decides the question.

**Bug-class note:** the fix suggestion below must be measured against the shipping verb
before it is accepted on this filing's say-so (PL-367).

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [ ] Filing posted to `framework:pickup` with `metadata.from_project=010-termlink` (so the T-2816 self-filter suppresses it and the ack-vs-safety conflict does not recur); the returned offset is recorded in this file as `FILED-AT: framework:pickup@<N>`
- [ ] Offset read back from the topic and confirmed to carry this filing — `delivered` from `channel post` means queued, not received (T-2876); assert on the reader, never the sender
- [ ] Filing names the DEFECT as the guess (not the path), quantifies it with the 11 misattributed filings, and cites PL-373 / `learnings.yaml:4418`
- [ ] Filing states the attribution-vs-addressing split and the git-worktree case that makes a path-derived attribution unsafe, so upstream can see why "just use the absolute path" does not close it
- [ ] Suggested fix is REFUSE-on-absent (an unknown label filters as unknown), explicitly not a third derived name
- [ ] No local edit to `.agentic-framework/` — `git status --porcelain .agentic-framework/` is empty (G-062)

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

### 2026-09-21T10:48:39Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3037-file-upstream-railprojectlabel-guesses-a.md
- **Context:** Initial task creation
