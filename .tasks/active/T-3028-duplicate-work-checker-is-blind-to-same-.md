---
id: T-3028
name: "Duplicate-work checker is blind to same-branch duplicates: all four axes exclude
  the base"
description: >
  scripts/check-task-id-collisions.sh (T-2800) builds its candidate set as IDs NOT
  already in the base (:168, :171) and runs all four axes (A colliding-ids, B near-duplicate-titles,
  C duplicate-new-files, D same-line-fix) over that set. Two tasks that both live
  on main are excluded by construction before any axis looks, so a duplicate filed
  on the same branch is invisible to every axis. Measured 2026-09-20: T-3018 was filed,
  scored, selected as arc-008's top Q1 item and started before anyone noticed it duplicated
  T-2950, closed 11 days earlier in the same arc by the same route. The checker run
  against the real tree that session reported 'no colliding IDs, no duplicate files
  (7 branches scanned against main)' — correct on its own terms, blind to the duplicate
  in front of it. Axis B's rare-word scorer would have fired on the two titles (shared
  rare terms: G-087-safe, budget, read, resume); it never got the chance. Sibling
  class to the August incident T-2800 was built for, arriving from the direction it
  does not cover.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: [scripts/check-task-id-collisions.sh]
related_tasks: [T-2800, T-2915, T-3018, T-2950]
arc_id: arc-008
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
# demo_target: true               # T-2286: optional — marks task as reserved for an orchestrated demo
#                                 # worker (e.g. arc-010 HM-A dispatches via mcp__fw__work_on). When set,
#                                 # `fw work-on T-XXX` refuses unless --i-am-demo-orchestrator (CLI) or
#                                 # FW_I_AM_DEMO_ORCHESTRATOR=1 (env) is passed. Prevents the parent
#                                 # session from consuming the captured→started-work transition the demo
#                                 # worker expects to drive. Origin OBS-057.
created: 2026-09-20T15:07:25Z
last_update: 2026-09-20T18:46:02Z
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
  - ts: '2026-09-20T15:09:40Z'
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
cost_estimate_proposed:
  - ts: '2026-09-20T15:09:59Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 2
      effort: 8
    rationale: blast_radius=1 (single-component); tier=2 (workflow:build); 
      effort=8 (lines=204,acs=4)
    rubric_sha: e4a00f38e801
---

# T-3028: Duplicate-work checker is blind to same-branch duplicates: all four axes exclude the base

## Context

**The blindness is real; two of the three claims in the `description:` above are not.**
That frontmatter is left as filed — it is the honest record of what was believed — but
measurement under AC1/AC2 corrected it on two counts, and the corrections changed the
remedy:

1. *"all four axes exclude the base"* — **false for axis D.** A/B/C do drop any id already
   in BASE (`if not i or i in base_ids` at :178, and axis C's `--diff-filter=A BASE...ref`
   at :273). Axis D does not: `sides = sorted(set(BRANCHES)) + [BASE]` (:339) puts main in
   deliberately (T-2915), and the real-tree run fires on `worktree-charter-review-2026-0814
   <-> main`, proving it. D is nonetheless blind here for a *different* reason — it pairs
   only DISTINCT sides, and inspects only `--diff-filter=M` files, so two task files added
   on the same side have no partner and are not even the right file status.
   The unifying cause is therefore sharper than base-exclusion: **every axis is a CROSS-SIDE
   comparison.** That is why widening the candidate set cannot fix it — both duplicates sit
   on the same side either way.

2. *"Axis B's rare-word scorer would have fired ... it never got the chance"* — **false.**
   Fed the two real titles against the real 2737-title corpus, they share `budget` (df=10),
   `read` (df=101), `safe` (df=12) and **zero** rare terms, where firing needs ≥2 at df≤4.
   "G-087-safe" never survives tokenisation (`g` is ≤2 chars, `087` is numeric). The scorer
   would have scored this pair 0 with full access.

**Disposition.** Three candidate detectors were measured and all three rejected: corpus-wide
axis B fires on 58 pairs and still scores the ground truth 0; component overlap is unusable
(T-2950 declares none; 62,554 corpus pairs share ≥1); a description-level variant only
"catches" the pair at threshold ≥2 via `invoking` (df=3) and `plausible` (df=4) — two
incidental prose adverbs unrelated to the defect — at 123 firing pairs, and misses it at every
principled threshold. Catching the right pair for the wrong reason is the T-2831 vacuous-check
class, so **no detector was shipped.**

What shipped instead is the honest half: the checker's own summary line read as a clean bill
over the task corpus when it is only a statement about cross-side NEW ids. Every output path
now declares that scope (T-2680 precedent, already the file's own convention for `--no-titles`).

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] Blindness is measured per-axis and located by line, not asserted from the filing. For each of A/B/C/D, the specific construct that prevents it from seeing two same-side tasks is cited; the filing's claim that "all four axes exclude the base" is TESTED, not inherited, and corrected if the code disagrees.
- [x] The ground-truth pair (T-3018 / T-2950) is fed to the real checker on the real tree and confirmed unreported; and axis B's rare-word scorer is fed those two real titles directly, to separate "the signal exists but is unreachable" from "there is no signal" — a guard's green is not evidence until it has been fed the violation it claims to catch (PL-328).
- [x] Disposition is decided on evidence with provenance BEFORE anything is filed or built: whether the remedy may land locally is settled by reading where the file actually lives (project-owned vs vendored, G-062), and any corpus-wide comparison is cost- and noise-measured on the real corpus before being committed to as the deliverable.
- [x] Whatever ships is proven load-bearing by a mutant, not by its own clean run: reverting the shipped change makes the fixture suite go RED, and restoring it returns it to GREEN. No permanently-red guard is left behind (T-2818/T-2833 fatigue trap).
- [x] No detector is shipped that only appears to work. If measurement shows no principled signal separates the ground-truth pair from corpus noise, that is RECORDED as the finding and no vacuous detector is built to satisfy a criterion — a guard that catches the right pair for the wrong reason is the T-2831 class, and shipping one here would reproduce inside the guard layer the exact defect this arc exists to catch. (This criterion replaces an earlier AC4 clause requiring the shipped artifact to fire on the ground-truth pair; that clause presumed a detector was the correct remedy, which the measurement under AC2/AC3 disproved. Recorded rather than silently reinterpreted.)

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

bash tests/task-id-collision-fixtures.sh > /tmp/.t3028-fix 2>&1 && grep -q ", 0 failed" /tmp/.t3028-fix
test "$(grep -c 'print("  scope: %s" % SCOPE_NOTE)' scripts/check-task-id-collisions.sh)" = "2"
test "$(grep -c '^SCOPE_NOTE = ' scripts/check-task-id-collisions.sh)" = "1"
grep -q '"scope": SCOPE_NOTE' scripts/check-task-id-collisions.sh
grep -q "mutant-scope-check" tests/task-id-collision-fixtures.sh
bash scripts/check-task-id-collisions.sh > /tmp/.t3028-real 2>&1 && grep -q "SAME side" /tmp/.t3028-real
test -z "$(git status --porcelain .agentic-framework/)"

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

- **The filing was right about the symptom and wrong about the cause, twice.** The checker
  genuinely cannot see this duplicate (real tree, rc=0, pair unreported). But "all four axes
  exclude the base" is false for axis D, which includes BASE deliberately and was observed
  firing against `main` in the same run; and "axis B would have fired" is false — the scorer
  gives the pair 0 rare terms against the real 2737-title corpus. Two greps and one scorer
  run settled both. Third task in this arc where measuring the recorded premise changed the
  work (T-3029's blocker was false, T-3030 flipped two-thirds).
- **The sharper cause forbids the obvious fix.** Base-exclusion is one mechanism of blindness
  (A/B/C); same-side pairing is another (D). The invariant is that *every axis is a cross-side
  comparison*, so widening the candidate set cannot help — both duplicates sit on the same
  side either way. Had the filing's cause been accepted, the fix would have been built,
  shipped, and still blind.
- **The strongest result was a refusal to ship.** A description-level detector DOES fire on
  the ground-truth pair at threshold ≥2 — via `invoking` (df=3) and `plausible` (df=4), two
  prose adverbs with no relation to the defect, at 123 firing pairs, and it misses at every
  principled threshold. That is a guard that catches the right pair for the wrong reason: it
  would have passed its own fixture, satisfied the original AC4 literally, and been worthless.
  Declining to build it is the T-2831 lesson applied *before* the vacuous check exists rather
  than after — which is the only time it is cheap.
- **An AC was amended mid-task, deliberately visibly.** The original AC4 required the shipped
  artifact to fire on the ground-truth pair; that presumed a detector was the right remedy,
  which AC2/AC3 disproved. Rather than silently reinterpret it — the "claim outran its check"
  disease inverted — it was replaced with an explicit criterion forbidding a vacuous detector,
  and the replacement says so in its own text.
- **Two gate refusals, both recorded, neither bypassed.** P-002 #16 on `while read` (known
  loop-keyword class). G-020 correctly blocked on placeholder ACs — but it blocked two
  *read-only* commands to get there: `sed -n '150,200p'` (no `-i`) and an awk program whose
  `NR>=140` contains `>`. Its write-pattern detector matches the quoted-`>` shape already
  recorded for P-002, so that shape now has a second host. Reshaped to `head|tail`; no --force.

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

### 2026-09-20T15:07:25Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3028-duplicate-work-checker-is-blind-to-same-.md
- **Context:** Initial task creation

### 2026-09-20T18:37:47Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)
