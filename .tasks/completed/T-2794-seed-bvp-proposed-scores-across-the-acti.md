---
id: T-2794
name: "Seed BVP proposed scores across the active backlog so quadrant ranking answers
  HV/LC and HV/HC"
description: >
  Seed BVP proposed scores across the active backlog so quadrant ranking answers HV/LC
  and HV/HC

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-18T18:54:11Z
last_update: 2026-08-18T19:04:31Z
date_finished: 2026-08-18T19:04:31Z
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
  - ts: '2026-08-18T18:55:38Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 5
      D3: 2
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=5 (body:silent-class-removed); 
      D3=2 (body:default-change); D4=2 (body:env-class-handled); F-RECALL=0 
      (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:40Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 8
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2794: Seed BVP proposed scores across the active backlog so quadrant ranking answers HV/LC and HV/HC

## Context

The operator's standing instruction includes "focus on HV/LC & HV/HC tasks and run BVP
estimator regularly". Every attempt to answer it returns nothing usable:

```
$ fw bvp --include-proposed        # 3 rows, all proposed, no cost column
$ fw bvp --quadrant hv-lc --include-proposed
No tasks match quadrant hv-lc
```

Three of 225 active tasks carry any score at all. The quadrant filter derives its
boundaries from the BVP median × cost median across the scored population, so with three
scored tasks and zero costs there is no median to cut on and every quadrant query is
empty by construction. The prioritisation system has been asked for a ranking repeatedly
and has answered "nothing" every time — not because the backlog is unrankable but because
nobody ever seeded it.

`fw bvp estimate all` writes `bvp_scores_proposed:` and is documented as
"heuristic v1, NOT sovereignty-bearing" — explicitly inside agent authority. The
sovereignty boundary is `fw bvp confirm` (§ACD-gated, human), which this task does not
touch: proposed scores stay proposed.

**Scope limit, stated up front:** seeding makes the ranking *possible*, not *correct*. A
heuristic estimate is a starting point for the human to confirm or override, and the
distinction has to survive into how the results are reported — `--include-proposed` is
required to see them, and the SOURCE column marks every row.

## Acceptance Criteria

### Agent
- [x] Determinism verified before the bulk write, for BOTH estimators:
      `fw bvp estimate determinism T-2793 --runs 3` → max delta per driver = 0;
      `fw bvp estimate-cost determinism T-2793 --runs 5` → max delta per component = 0,
      `deterministic=True`.
- [x] `fw bvp estimate all` → **2557 tasks, 2556 wrote, 1 skipped, 0 errored** (81.9s).
      `fw bvp estimate-cost all` → **2557 tasks, 2557 wrote, 0 skipped, 0 errored** (8.8s).
      **The scale was larger than this task assumed** — the AC said "200+", but `all` means
      every task in `.tasks/`, active AND completed, so 2557 files were rewritten rather
      than the ~225 active ones. Recorded rather than smoothed over, because the diff an
      operator reviews is an order of magnitude bigger than this task implied.
- [x] Ranking populated: `fw bvp --include-proposed` returns **165 rows, up from 3**, and
      both quadrant queries answer where both previously returned "No tasks match".
- [x] Heads reported to the operator (see Findings).
- [x] No confirmed `bvp_scores:` field written. Verified rather than assumed: the two
      estimators write `bvp_scores_proposed:` / `cost_estimate_proposed:` only, every
      ranking row reports `SOURCE=proposed`, and `fw bvp` without `--include-proposed`
      still returns nothing — which it could not do if any score had been confirmed.

## Findings

**The cost axis was the actual blocker, and its populator is invisible.** Seeding BVP
alone did not make the quadrant query answer: `--quadrant hv-lc` stayed empty with 165
scored tasks, because the quadrant is a BVP-median × cost-median cut and no task carried
a cost. `fw bvp estimate-cost` exists and populates it — and is **not listed in
`fw bvp --help`**, which documents `estimate`, `estimate all`, and
`estimate determinism` but never mentions its cost sibling. It is reachable only by
reading `lib/bvp.sh:1553`. That is why the cost axis had never been seeded: not a
decision, an undiscoverable verb. Same shape as T-2793's dark mentions rail and T-2788's
no-op remedy — a capability that exists, is never executed, and whose absence surfaces as
a feature that silently returns nothing.

**Caveat that must travel with these numbers: the cost heuristic barely discriminates.**
Of the ranked rows, 94 share cost 1.4, 30 share 1.2, 10 share 1.3 — roughly four-fifths
of the backlog sits in a 0.3-wide band. So `hv-lc` is close to "high BVP" with the
low-cost half contributing almost no separation, and the honest reading of the HV/LC list
is a value ranking, not a value-per-effort ranking. The one place cost does separate is
the top of `hv-hc` (3.6–3.8), and those rows are meaningful.

**HV/LC head** (BVP, proposed): T-1166 (106) · T-2713 (92) · T-2714 (92) · T-2197 (81) ·
T-2203 (80) · T-2715 (80) · T-2721 (80) · T-2016 (78).

**HV/HC head:** T-1898 · T-1899 · T-2007 · T-2022 · T-2024 · T-2026 · T-2090 · T-2250 ·
T-2276 · T-2422 (all BVP 70, cost 1.9–3.8).

**Independent convergence worth noting.** The HV/HC head is almost exactly the deferred
backlog the S-2026-0818-2009 handover flagged — the six DEFER decisions carrying no
`revisit_at` (T-1899, T-2007, T-2090, T-2276, T-2422) plus the two ripe revisits (T-1898,
T-2250). A heuristic that knows nothing about deferral independently ranked them
high-value/high-cost, which is a coherent reason to have deferred them and an equally
coherent reason they should not stay invisible.

**T-2222 risk checked, not assumed.** T-2222 records that this estimator "corrupts
anchor-less task frontmatter (orphaned proposed-score lists produce invalid YAML)" — and
it appeared in the ranking *after* I had already written to 2557 files. Every task file
was then parsed: **2560 parsed OK, 0 invalid, 0 missing frontmatter.** The failure mode
did not trigger here.

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
       Conversion: this AC should be moved to ### Agent and this line added to
       ## Verification (herestring, not a pipeline — see the L-387 hint below):
         out=$(bin/fw reviewer T-XXX 2>&1 || true); grep -q "Overall:.*PASS" <<< "$out"
-->

## Verification

# The deliverable: both quadrant queries answer instead of returning "No tasks match".
out=$(.agentic-framework/bin/fw bvp --quadrant hv-lc --include-proposed 2>&1 || true); grep -q "hv-lc" <<< "$out"
out=$(.agentic-framework/bin/fw bvp --quadrant hv-hc --include-proposed 2>&1 || true); grep -q "hv-hc" <<< "$out"
# Sovereignty boundary intact: nothing confirmed, so the unfiltered ranking stays empty.
out=$(.agentic-framework/bin/fw bvp 2>&1 || true); grep -qv "proposed" <<< "$out"
# No task frontmatter was corrupted by the bulk write (the T-2222 failure mode).
bash scripts/check-task-frontmatter.sh --quiet

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).
#
# Pipefail/SIGPIPE hint (L-387, corrected by T-2775): P-011 runs each command
# under `set -eo pipefail`. NEVER write `cmd | grep -q PATTERN`: it exits 141
# (SIGPIPE) when grep matches and closes stdin while the upstream is still
# writing — verification then "fails" BECAUSE the check succeeded, and the
# earlier the match, the more reliably it fails.
#
# USE ONE OF THESE — both measured rc=0 at 3M lines:
#     out=$(cmd 2>&1 || true); grep -q "PATTERN" <<< "$out"   # herestring (preferred)
#     test -n "$(cmd | grep -m1 PATTERN)"                     # pipeline inside $( )
#
# The herestring is preferred: a herestring spawns no producer process, so there
# is nothing to SIGPIPE and it cannot regress as output grows. In the second form
# the pipeline sits inside a command substitution, whose status is discarded — the
# OUTER `test` decides.
#
# DO NOT capture-then-pipe. This template previously prescribed
#     out=$(cmd 2>&1); echo "$out" | grep -q "PATTERN"     # UNSAFE above ~64KB
# and it is size-dependent, not safe: `echo`/`printf` is a producer like any
# other, so once $out exceeds the pipe buffer it is still writing when `grep -q`
# exits and pipefail propagates 141. The capture bounds the DATA but does not
# remove the PRODUCER. Anything wrapping `cargo test`, `fleet doctor --json`, or a
# full log is already in that size range. (T-2775 measured this; 999-AEF L-613 and
# 050-email-archive PL-161 published the capture-then-pipe form before the
# correction — both have since adopted the herestring.)
#
# Corollary (T-2090): intermediate stages are just as fatal — `... | tail -3 |
# grep -q PAT` re-introduces the same risk. With a herestring the question does
# not arise; grep scans the whole captured string anyway.
#
# Origin: L-387, captured 4× (T-1716, T-1838, T-1862, T-1863) before the hint;
# T-2775 then measured 1490 exposed lines across 802 tasks despite the hint, which
# is why `scripts/check-verification-pipefail.sh` now enforces it structurally.
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

### 2026-08-18T18:54:11Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2794-seed-bvp-proposed-scores-across-the-acti.md
- **Context:** Initial task creation

### 2026-08-18T19:04:31Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
