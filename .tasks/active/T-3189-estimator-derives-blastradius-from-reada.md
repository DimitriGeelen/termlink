---
id: T-3189
name: "Estimator derives blast_radius from readable task evidence instead of leaving
  85% uncosted"
description: >
  Estimator derives blast_radius from readable task evidence instead of leaving 85%
  uncosted

status: started-work
workflow_type: build
owner: agent
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
created: 2026-09-27T21:20:40Z
last_update: '2026-09-27T21:34:09Z'
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
cost_estimate_proposed:
  - ts: '2026-09-27T21:34:09Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 5
      tier: 2
      effort: 8
    rationale: blast_radius=5 (4-file-refs-derived-T-3189); tier=2 
      (workflow:build); effort=8 (lines=322,acs=13)
    rubric_sha: e4a00f38e801
---

# T-3189: Estimator derives blast_radius from readable task evidence instead of leaving 85% uncosted

## Context

The quadrant system that an autonomous run selects work by is computed over **6 of 248
tasks**. Operator decision 2026-09-27 (SQ-2), after a strawman/steelman scored against the
four Constitutional Directives: **107/120 for deriving the value from evidence, 48/120 for
defaulting it in the template.** The whole 59-point margin is D1+D2 (76 vs 16).

**Measured before starting**, using the ranker's OWN predicates (`compute_cost`,
`_proposal_is_all_no_signal`) over its own corpus, per PL-386:

| workflow_type | in corpus | all-no-signal | has BVP signal |
|---|---:|---:|---:|
| build | 202 | **0** | 202 |
| inception | 35 | **35** | 0 |
| test/refactor/design/decommission | 11 | 0 | 11 |

Of 248 tasks, 37 have a cost; 31 of those are excluded as all-no-signal; **6 survive**. The
partition is exact: **31 costed inceptions via `target_blast_radius`, 6 costed builds via
`components:`, zero by any other route.** The two filters are perfectly anti-correlated —
the tasks that have a cost are the ones with no BVP signal, and vice versa. The 6 survivors
are builds that acquired `components:` by having been *worked on*, which is the trap:
`components:` is resolved from git at the `work-completed` transition, i.e. after the point
where the cost could have informed the decision to start.

Two consequences that set this task's scope:

1. **The no-signal exclusion is not a second blocker for builds** (0 of 202). So population
   is the only thing left, and the ceiling is **6 → up to 213**, not 6 → 37.
2. T-3188 opened the `target_blast_radius` path for every workflow type and left it
   deliberately unpopulated — correct and inert, because only the inception template ships
   the field (inception 32/36, build 1/279). This task is the population half, and it must
   not become the strawman T-3188 rejected: **a default is not an estimate.**

Precedence is the load-bearing property, in this order and no other:

    components:  (measured from git)   >  target_blast_radius:  (declared)  >
    derived from body evidence          >  UNMEASURED (None)

A derivation must never override either a measurement or an author's explicit declaration,
and must return `None` rather than manufacture a value — the T-3105 distinction ("could not
measure" ≠ "measured and empty") and PL-371 ("BVP no-signal defaults outrank measured
work"), on the cost axis.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] The signal is **chosen by measurement, not assumption**: coverage of the candidate
      body-evidence signal across the real 202-build corpus is measured and the figure is
      recorded in `## Evolution` before the scorer is written.
      → four candidates measured; the intuitive one (Verification-only, 16%) lost to
      body-wide resolving paths + unique bare names (51%). Recorded in `## Evolution`.
- [x] `score_blast_radius` returns a derived value with an evidence string naming the
      derivation, for a build task carrying neither `components:` nor
      `target_blast_radius:` but with readable file evidence in its body.
      → `→3 (2-file-refs-derived-T-3189)`. Case 1, incl. all five non-inception types.
- [x] **Precedence holds at every boundary**, pinned by fixtures: `components:` beats
      `target_blast_radius:` beats derived; derived never overrides either.
      → Case 2, both boundaries, incl. a measured single component beating a declared 9.
- [x] **The derivation cannot manufacture a value.** A task with nothing readable still
      returns `None` with the UNMEASURED evidence string — no default, no floor.
      → Case 3 (prose, unresolvable paths, ambiguous bare names) + Case 7 (empty index).
- [x] Evidence strings are attributable per path, so a reader can distinguish measured from
      declared from derived without opening the task.
      → `single-component` / `target_blast_radius:build-T-3188` / `…-derived-T-3189`.
- [x] Malformed / adversarial input (non-list `components:`, junk `target_blast_radius:`,
      body with no paths) falls through without raising and without defaulting.
      → Case 6, four legs, none raises.
- [x] Fixture suite `tests/bvp-derived-blast-radius-fixtures.sh` exists, carries the
      `# guard-layer: source` marker, and passes with 0 failures — including a harness
      control that exits 2 if the scorer is unreachable, and a **behavioural**
      pre-change ground-truth case (run each historical version, do not text-match).
      → **34 assertions, 0 fail.** Case 0 exits 2 on an unreachable scorer; Case 8
      behaviourally recovered `f8a4160c1` as the pre-change revision.
- [x] Before/after quadrant population is measured against the live ranker and recorded.
      If the number does not move, it is reported as FAILED, not rewritten to pass.
      → **6 → 111 tasks** (211/248 uncosted → 102/248). It moved, so this passes on
      measurement — the criterion T-3188 had to mark FAILED.
- [x] `.vendor-divergence.yaml` registers this divergence (estimator.py is vendored,
      G-062) and `bash scripts/check-vendor-divergence.sh` reports all changes classified.
      → 31 commits touch vendored code, all registered.
- [x] Filed upstream at `framework:pickup`, verified by read-back (T-2876: delivered ≠
      received; body arrives base64 under `payload_b64`).
      → offset 210, sha256 `53b6c83922366eb1`, 7163 bytes sent and read back identical.
      The post itself reported only `delivered-unconfirmed`, which is why the read-back
      is the evidence and the post is not.

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

- [ ] [REVIEW] The repopulated cost axis produces a sane ranking
  **Steps:**
  1. `cd /opt/termlink && .agentic-framework/bin/fw bvp --include-proposed --quadrant hv-lc`
  2. Read the top ~10 rows. Ask of each: does a derived cost this cheap/expensive match
     what you know the task actually involves?
  **Expected:** The HV-LC list reads like plausible next work, not like an artifact of the
  derivation. Specifically: no task appears cheap *because* its body happens to name few
  files while the work itself is broad.
  **If not:** Say which task is mis-costed and what it should be. The derivation is a
  heuristic over task prose; a systematic skew is a scorer bug, and a one-off is a task
  whose body under-describes its own reach — the two need different fixes, and only you
  can tell them apart here. This is the axis autonomous selection runs on, so a skew
  compounds silently.

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
# ── Asserting an ABSENCE: prove the search could have succeeded (T-3144) ──
#
# `! grep -q "PATTERN" file` exits 0 when the pattern is absent. It ALSO exits 0
# when the file was renamed, deleted, or is empty — so the leg cannot distinguish
# "the bad thing is not there" from "I could not look", and the gate reports green
# over a check that never ran. Pair every absence assertion with something that
# fails if the search could not happen:
#
#     test -f path/to/file && ! grep -q "PATTERN" path/to/file    # existence first
#     grep -q "KNOWN_MARKER" f && ! grep -q "PATTERN" f           # positive companion
#     cmd > /tmp/.out 2>&1 && ! grep -q "PATTERN" /tmp/.out       # &&-joined producer
#
# Count-equals-zero is the same defect wearing a different hat, and it is the one
# that bites hardest over a COMMAND's output rather than a file:
#
#     [ "$(cargo clippy --workspace 2>&1 | grep -c "^error")" = "0" ]   # WRONG
#
# If cargo is missing, or dies before emitting diagnostics, there are no `^error`
# lines, the count is 0, and the leg passes — a build gate that goes green
# precisely when the build could not run. Measured in this corpus, not invented.
# Keep the producer's exit code in the verdict:
#
#     cargo clippy --workspace > /tmp/.out 2>&1 && ! grep -q "^error" /tmp/.out
#
# T-3144 censused 2853 task files: 71 absence assertions, 41 already correct, 30
# not. The convention mostly works — this note is here so the next one is written
# right, because a vacuous leg is invisible until the day the path moves.
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

# The fixture suite is the primary gate. It carries its own harness control (exit 2 when
# the scorer is unreachable), so a suite that could not run cannot report green.
bash tests/bvp-derived-blast-radius-fixtures.sh > /tmp/.t3189-fix.out 2>&1 && grep -q "^Fail: 0" /tmp/.t3189-fix.out

# T-3188's suite must still pass — precedence between components: and target_blast_radius:
# is the property this task is most likely to break.
bash tests/bvp-target-blast-radius-fixtures.sh > /tmp/.t3189-prev.out 2>&1 && grep -q "^Fail: 0" /tmp/.t3189-prev.out

# G-062: estimator.py is vendored. Every local change must be classified.
bash scripts/check-vendor-divergence.sh > /tmp/.t3189-vd.out 2>&1

# The scorer must still be syntactically loadable by the real interpreter, not just by the
# fixture harness (a SyntaxError here would make every estimate silently unavailable).
python3 -c "import ast,sys; ast.parse(open('.agentic-framework/agents/termlink/bvp-estimator/estimator.py').read())"

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

### 2026-09-27 — the signal was chosen by measurement, and the first three candidates lost

- **What changed:** AC1 required measuring the candidate signal before writing the scorer,
  and that ordering paid for itself three times.
  Candidates measured across the 210 in-corpus build tasks:

  | candidate | coverage |
  |---|---:|
  | paths in `## Verification` only | 34/210 (16%) |
  | repo-prefixed paths anywhere in body, resolving on disk | 86/210 (40%) |
  | + bare filenames resolving **uniquely** across the repo | 109/210 (51%) |
  | final, with section-aware comment stripping | **107/210 (51%)** |

  Verification-only was the intuitive choice and the weakest. Bare names rescued 23 tasks
  that name a file without its directory, and the uniqueness requirement is what keeps that
  from being noise — it correctly rejects `README.md`, `Cargo.toml`, `lib.rs`, `__init__.py`.
- **Plan impact:** none to the decision; the steelman predicted "coverage arrives gradually"
  and 51% is that prediction landing. ~100 tasks name no files at all and stay UNMEASURED,
  which is the correct answer for them rather than a shortfall to engineer away.
- **Triggered:** nothing; the scope fence held.

### 2026-09-27 — the template-comment trap, which nearly shipped

- **What changed:** an intermediate measurement read **205 of 210** tasks as costed. That
  was not success, it was the defect. `.claude/settings.json` appears in **177 of 210**
  task bodies — inside a `#` shell comment in `## Verification` (the template's L-398
  enforcement-baseline hint). Stripping HTML comments alone leaves **98 tasks acquiring a
  cost derived entirely from boilerplate they never touch**, and nothing about the output
  looks wrong.
- **Plan impact:** the strip had to become **section-aware**. Inside `## Verification` a
  `#`-leading line is a shell comment (P-011's own rule); everywhere else it is a markdown
  heading. The two are textually identical, so a global strip breaks headings and a global
  keep fabricates costs. Both directions are pinned by Case 4.
- **Triggered:** flagged as the part most worth carrying upstream (offset 210), because any
  consumer using the shipped template has this identical boilerplate path — so a naive
  implementation of this same idea fabricates costs at scale *and looks like it is working*.

### 2026-09-27 — a regex that could not see the guard allowlists

- **What changed:** Case 5 failed on first run. The path pattern required a dotted filename,
  so every extensionless file was unreachable — including `.context/checks/*-allowlist`,
  the guard ledgers this repo's tasks genuinely do modify.
- **Plan impact:** the extension requirement was dropped for **full paths only**. Precision
  never came from the shape — it comes from the `m in tracked` test — so a relaxed shape
  admits more candidates and exactly as many answers. Bare names still require an extension,
  because with no directory segment to anchor them an extensionless token is any word.
- **Triggered:** nothing. Found by a fixture written before the code was believed finished.

### 2026-09-27 — measured false positives, kept rather than tuned away

- **What changed:** 2 of 256 tasks (T-3095, T-3096) derive their only reference from
  `.agentic-framework/bin/fw`, which they **invoke** rather than modify — and both land at
  the top of hv-lc, i.e. exactly what an autonomous run picks up first.
- **Plan impact:** inspected individually rather than patched. Each has a ~10KB body
  discussing the handover generator at length **without naming one handover source file by
  path**. So the derivation is accurate to its input and those two tasks under-describe
  their own reach. A `bin/fw` special case would be the kind of tuning that rots, and it
  would hide a real signal: the cost axis now rewards naming the files you will touch.
- **Triggered:** the operator-review Human AC, which is where a judgement about skew
  belongs. The false-positive floor is recorded as a measured number, not an impression.

### 2026-09-27 — result

- **What changed:** before, 211/248 uncosted with quadrant thresholds over **6** tasks.
  After, 102/248 uncosted with thresholds over **111** (42 hv-lc, 38 lv-lc, 16 lv-hc,
  15 hv-hc). 120 task files rewritten by `cost-all`. T-3189 scored itself at
  `blast_radius: 5 (4-file-refs-derived-T-3189)`.
- **Plan impact:** this is the AC that FAILED on T-3188 and passes here on measurement —
  that task opened the path and this one put traffic on it.
- **Triggered:** nothing outstanding.

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

### 2026-09-27T21:20:40Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3189-estimator-derives-blastradius-from-reada.md
- **Context:** Initial task creation
