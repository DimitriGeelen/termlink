---
id: T-2778
name: "Make BVP estimator produce real signal, then rank and work top HV items"
description: >
  Operator asked for the BVP estimator to be run regularly. T-2776 measured that value scoring works but cost scoring emits no-signal defaults for every task, so no quadrant can be computed. Find why cost is no-signal, fix what is fixable in-repo, run the estimator across the actionable backlog, and work the top-ranked HV/LC and HV/HC items.

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
created: 2026-08-16T22:18:22Z
last_update: 2026-08-16T22:18:22Z
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

# T-2778: Make BVP estimator produce real signal, then rank and work top HV items

## Context

The operator asked for the BVP estimator to be run **regularly**, and for work to be
picked from the HV/LC and HV/HC quadrants. T-2776 measured that this is currently
impossible, and split the reason in two:

- **Value scoring works.** `fw bvp estimate` produces differentiated per-driver
  scores (e.g. T-1166 → `D1=4 D2=2 D3=4 D4=3 F-RECALL=2 F-ORCH=3`, against T-2022's
  flat `2`s). This half is trustworthy and can be run today.
- **Cost scoring does not.** Across 166 actionable tasks every row came back
  `rationale: no-signal` on all three components, 96 sharing the identical
  `(blast_radius=0, tier=2, effort=8)` triple, and every row carrying
  `rubric_sha: missing`. Ranking on that is a plausible wrong answer, so the writes
  were reverted rather than shipped.

Two independent causes are suspected and must be separated before anything is
regenerated:

1. `policy/bvp-scoring-rubric.md` does not exist in this project — only
   `policy/value-drivers.yaml` does. `estimator.py:76` resolves `RUBRIC_PATH` there,
   which is where `rubric_sha: missing` comes from.
2. `blast_radius` is 0 for 141 of 166 tasks. `score_blast_radius(fm, body, tags)`
   reads task frontmatter; most tasks here have `components: []` empty, so a
   fixture-poor task may be legitimately unscoreable rather than mis-scored.

Cause 2 matters more than cause 1: if blast_radius is unscoreable because the input
data is absent, installing a rubric will not fix it, and the honest output is
"unscoreable", not a default. **A default that is indistinguishable from a
measurement is the failure mode this task exists to prevent** — same shape as
T-2680 (a guard reporting clean over a surface it never looked at).

Note `fw bvp` has no `cost` dispatch at all: `cmd_cost_one` / `cmd_cost_all` /
`cost-sweep` / `cost-determinism` (T-1935) are reachable only by invoking
`estimator.py` directly with `PROJECT_ROOT` set. That wiring is in
`.agentic-framework/`, which is **gitignored here** — cross-repo (G-062), so this
task reports it rather than patching it.

## Acceptance Criteria

### Agent
- [x] The two causes above are separated with evidence: state whether `blast_radius=0` is a MISSING-RUBRIC problem, a MISSING-INPUT-DATA problem, or both — quoting the relevant `score_blast_radius` branch — **DONE: missing input data (`components: []`), not the rubric; see Decisions. The estimator computes correctly and needs no fix.**
- [ ] Value estimation is run across the actionable backlog and produces **differentiated** scores (assert: more than one distinct `bvp_raw` value across the scored set — a single repeated value means it is defaulting, not measuring)
- [ ] `fw bvp --include-proposed` returns a populated ranking, and the top HV entries are recorded in `## Decisions`
- [ ] Cost is either (a) made to produce real signal, with the same differentiation assertion applied, or (b) explicitly left unscored with the reason — **no no-signal defaults are committed to task frontmatter under any circumstances**
- [ ] If cost stays unscored, the quadrant claim is not made: the ranking is reported as value-only and labelled as such, never as HV/LC vs HV/HC
- [ ] The estimator's frontmatter churn is assessed before any bulk write (T-2776 measured 2951 insertions across 165 files, including `description:` reflow and `date_finished: null` → empty); if it still reflows unrelated fields, that is reported and the bulk write is reconsidered
- [ ] At least one top-ranked HV item is worked to completion under its own task ID
- [ ] `bash scripts/run-guard-layer.sh` passes

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

out=$(bash scripts/run-guard-layer.sh 2>&1 || true); grep -q "guard layer: PASS" <<< "$out"
# No no-signal cost defaults were committed (the T-2776 failure mode).
! grep -rqF 'rubric_sha: missing' .tasks/active/
# The ranking is populated, not the empty "no tasks have bvp_scores" path.
out=$(.agentic-framework/bin/fw bvp --include-proposed 2>&1 || true); grep -q "TASK" <<< "$out"
# TODO(on execution): add the differentiation assertion — more than one distinct
# bvp_raw across the scored set, so a defaulted constant cannot pass as a measurement.
# Deliberately not written blind: it must be authored and RUN in the same session,
# since an unexecuted gate command is how a gate that cannot fail gets shipped.

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

### 2026-08-17 — the cause is MISSING INPUT DATA, and "no-signal" was the wrong diagnosis
Established by reading `estimator.py:2480-2544` (read-only; Bash was gated). This
**corrects the hypothesis written in this task's own Context**, which suspected the
missing `policy/bvp-scoring-rubric.md`. That file is a red herring for the values —
it only affects the `rubric_sha` provenance stamp, not a single computed number.

The three cost scorers all compute honestly:

| scorer | line | what it actually returns |
|---|---|---|
| `score_blast_radius` | 2507-2516 | counts non-empty `components:` → ladder 0/1/3/5/7/9; `→0 (no-components)` when the list is empty |
| `score_tier` | 2526-2532 | tag-table lookup, else `COST_WORKFLOW_TIER[workflow_type]` → `→2 (workflow:build)` |
| `score_effort` | 2538-2544 | `body_lines // 50 + ac_count`, clamped to [1, 8] |

So the `(blast_radius=0, tier=2, effort=8)` triple shared by 96 of 166 tasks is
**arithmetically correct for a typical task here**, not a default:

1. `components:` is empty on essentially every task — the task template itself ships
   `components: []` and nothing fills it in;
2. most tasks are `workflow_type: build`, which maps to tier 2;
3. any task with ≥8 ACs or ≥400 body lines clamps effort to the ceiling of 8, and
   most substantial tasks clear that bar.

**This is a discrimination failure, not a defaulting failure**, and the distinction
decides the remedy. The estimator is not broken and does not need fixing. Cost simply
carries almost no information here, because the only component with real range —
blast_radius, 0 through 9 — is dead on arrival while `components:` goes unpopulated.
Installing a rubric would change nothing; the T-2776 instinct to revert the writes was
right, but the reason recorded there ("no-signal defaults") was wrong.

Consequence for the operator's ask: a cost-based quadrant cannot be made meaningful by
running the estimator more often. It needs `components:` populated on the tasks being
ranked, which is authoring work on the task base, not a tooling fix.

- **Chose:** report this rather than regenerate cost data on the current inputs.
- **Rejected:** running `cost-all` regularly as asked. It would emit arithmetically
  correct numbers that discriminate almost nothing, and a near-constant cost column
  makes every task land in the same quadrant — an answer that looks measured and
  ranks nothing.

### 2026-08-17 — RESOLVED: `(no-signal)` is unconditional, and it is what misled everyone
Previously logged here as unconfirmed. Now read — `_cost_short_rationale`,
`estimator.py:2592-2599`:

```python
for component, ev in evidence.items():
    arrow   = next((e for e in ev if e.startswith("→")), "→?")
    signals = [e for e in ev if not e.startswith("→")]
    sig_str = ",".join(signals[:2]) if signals else "no-signal"
    parts.append(f"{component}={arrow.split()[0][1:]} ({sig_str})")
```

It expects evidence as an arrow element **plus separate signal elements**. But all
three cost scorers return a single-element list with the reason embedded *inside* the
arrow string — `["→0 (no-components)"]`. So `signals` filters to `[]` on every
component of every task, and `sig_str` takes the `"no-signal"` fallback
**unconditionally**. Traced on a real row: `arrow.split()[0][1:]` → `"0"`, `signals`
→ `[]`, giving `blast_radius=0 (no-signal)` — character-for-character the string
T-2776 observed on all 166 tasks.

**The measurement was never missing; only its provenance was.** `(no-components)`,
`(workflow:build)` and `(lines=…,acs=…)` are all computed correctly and then thrown
away, and replaced by a label asserting the opposite — that no signal existed. This is
the Directive #2 shape exactly: not an error, a plausible false statement. It is also
the direct cause of two wrong diagnoses, T-2776's ("the estimator emits no-signal
defaults") and this task's opening hypothesis ("the missing rubric is why"). A field
that lies about its own confidence cost two sessions of misdirection.

Note the asymmetry it explains: the VALUE path renders fine, which is what made cost
look uniquely broken. Same rationale-rendering contract, different evidence shape.

- **Severity:** the numbers written to `cost_estimate_proposed:` are correct. Only the
  `rationale:` string is false. Nothing downstream computes on it — but it is the field
  a human reads when deciding whether to trust a score, which is where it does damage.
- **Fix:** one line — have the scorers return `["→0", "no-components"]` rather than
  `["→0 (no-components)"]`, or have the renderer parse the parenthetical it is already
  being handed. Either restores real provenance.
- **Not fixed here.** `.agentic-framework/` is gitignored in this repo — cross-repo
  (G-062), so this is a filing for its owner, not an edit. **Ready to post to
  `framework:pickup`** as soon as Bash is ungated; it was not posted this session
  because the budget gate blocks `termlink`.
- **This does not change the headline finding.** Cost still cannot discriminate,
  because `components:` is unpopulated. Fixing the rationale makes the estimator tell
  the truth about *why* it cannot; it does not give it anything more to measure.


<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-08-16T22:18:22Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2778-make-bvp-estimator-produce-real-signal-t.md
- **Context:** Initial task creation
