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
- [x] Value estimation is run across the actionable backlog and produces **differentiated** scores (assert: more than one distinct `bvp_raw` value across the scored set — a single repeated value means it is defaulting, not measuring) — **DONE: 224 tasks scored, 47 distinct `bvp_raw`, range 0.0–107.0, 0 errored. Assertion authored and RUN, not written blind.**
- [x] **[AMENDED — mechanism changed by AC 6, substance unchanged]** A populated ranking is produced and the top HV entries are recorded in `## Decisions` — **DONE via `fw bvp estimate --dry-run --json` across all 224 active tasks, NOT via `fw bvp --include-proposed`.** The original wording named `--include-proposed`, which reads persisted `bvp_scores_proposed:` and therefore requires the bulk write that AC 6 below explicitly instructs to reconsider — and which was rejected. `fw bvp --include-proposed` currently returns 3 rows (all `COST: -`, `QUAD: -`), so the literal mechanism cannot satisfy the intent without the write. The dry-run path yields the identical numbers with zero writes. The assertion was not weakened; only the read path changed. Recorded so the amendment is visible rather than silent.
- [x] Cost is either (a) made to produce real signal, with the same differentiation assertion applied, or (b) explicitly left unscored with the reason — **no no-signal defaults are committed to task frontmatter under any circumstances** — **DONE: option (b). Cost left unscored; reason is unpopulated `components:`, established with evidence in `## Decisions`. Zero cost values written to any task.**
- [x] If cost stays unscored, the quadrant claim is not made: the ranking is reported as value-only and labelled as such, never as HV/LC vs HV/HC — **DONE: the ranking is labelled value-only in `## Decisions` and in the session report.**
- [x] The estimator's frontmatter churn is assessed before any bulk write (T-2776 measured 2951 insertions across 165 files, including `description:` reflow and `date_finished: null` → empty); if it still reflows unrelated fields, that is reported and the bulk write is reconsidered — **DONE: it still reflows. Measured 23+/3- for ONE task, only 15 lines being the wanted payload (~35% unrelated churn: `description:` reflow, `date_finished: null` → bare, `last_update:` re-quoted). Bulk write REJECTED. Cross-references existing T-2222.**
- [x] At least one top-ranked HV item is worked under its own task ID (not under T-2778) — **T-2644 selected and worked; see `## Decisions` for selection rationale.**
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
# No cost values of ANY kind were written to task frontmatter — this is the
# AC-4 option-(b) promise ("cost explicitly left unscored") made mechanical.
test -z "$(grep -l '^cost_estimate_proposed:' .tasks/active/*.md 2>/dev/null)"
test -z "$(grep -l '^cost_estimate:' .tasks/active/*.md 2>/dev/null)"
# The estimator emits a real per-driver payload, not an empty/erroring envelope.
out=$(.agentic-framework/bin/fw bvp estimate T-1166 --dry-run --json 2>&1 || true); grep -q '"D1"' <<< "$out"
# DIFFERENTIATION ASSERTION (the AC's core claim, now authored AND run).
# More than one distinct score-sum across a fixed 5-task sample, so a defaulted
# constant cannot pass as a measurement. Load-bearing: verified by negative
# control — feeding the SAME task 3× yields 1 distinct value and this exits 1.
# The pipelines sit inside $( ), whose status is discarded, so the outer `test`
# decides and SIGPIPE cannot decide it (T-2775).
test "$(for t in T-1166 T-2022 T-2644 T-2695 T-2713; do .agentic-framework/bin/fw bvp estimate $t --dry-run --json 2>/dev/null | python3 -c 'import sys,json;print(sum(json.load(sys.stdin)["scores"].values()))'; done | sort -u | wc -l)" -gt 1

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

### 2026-08-17 — value scoring is differentiated; the ranking is VALUE-ONLY and labelled as such
Ran `fw bvp estimate <id> --dry-run --json` across all 224 active tasks (no writes).
Weights from `policy/value-drivers.yaml`: `D1=9 D2=7 D3=5 D4=3 F-RECALL=6 F-ORCH=5`.

**Differentiation assertion — PASS.** 47 distinct `bvp_raw` values across 224 scored
tasks, range `0.0 .. 107.0`, 0 errored. The modal value (52.0) accounts for 66 tasks
(29%), so there IS clustering, but a defaulted constant would have produced exactly
one distinct value. This is measuring, not defaulting — the assertion the AC demanded
was authored and RUN in the same session, not written blind.

Filtered to genuinely actionable (`status != work-completed` AND `horizon in {now,next}`):
**125 of 224**. Excluded: 58 work-completed-but-still-in-`active/`, 41 horizon-later.
Owner split of the actionable set: 81 agent, 44 human.

**The quadrant claim is NOT made.** Cost is left unscored (option (b) of the AC), for
the reason established above: `components:` is unpopulated, so `blast_radius` is 0 for
essentially every task and the cost axis cannot discriminate. Per the AC, this ranking
is therefore reported as **value-only** and must never be described as HV/LC vs HV/HC.
No no-signal defaults were committed to any task's frontmatter.

Top actionable, agent-owned, by value:

| bvp_raw | task | what |
|---|---|---|
| 106 | T-1166 | retire legacy event.broadcast + inbox + file.send/receive |
| 92 | T-2713 / T-2714 | hook telemetry counts intentional blocks as failures / audit ignores core.hooksPath |
| 80 | T-2715 / T-2721 | worktree-safe hook-path resolution |
| 78 | T-2016 | fw upgrade replay drops flags |
| 78 | T-2644 | PTY interactive attach silently drops keystrokes on failed inject |
| 76 | T-2687 | MCP termlink_topics silently partial inventory |
| 75 | T-2695 | session-selftest proves exec only; charter also claims inject |

(T-2778 itself scored 107 — highest of all. Disregarded: ranking your own meta-task
first is an artifact, not a finding.)

**Selected T-2644** to work under its own task ID. Rationale: it is the highest-value
actionable item that is (a) a genuine product defect rather than a framework/meta item,
(b) in the charter's founding verb — "control terminal sessions", (c) squarely the
Directive #2 silent-failure class, and (d) session-sized. T-1166 at 106 is a multi-part
retirement arc, not session-sized; T-2713/14/15/21 are hook/audit items.

### 2026-08-17 — bulk write REJECTED: the estimator still reflows unrelated frontmatter
AC 6 required assessing churn *before* any bulk write. Probed by running a real (non-dry)
`fw bvp estimate T-2644`, measuring the diff, then reverting to clean. Result: **23
insertions, 3 deletions for ONE task**, of which only 15 lines are the wanted
`bvp_scores_proposed:` payload. The other 8 are unrelated:

- `description:` is **reflowed** — the folded scalar is re-wrapped from 1 line to 6.
  Semantically identical YAML, textually a rewrite of a field BVP scoring never touches.
- `date_finished: null` → `date_finished:` (bare). Both parse as null; the text changes.
- `last_update:` is bumped *and* re-quoted (`2026-...Z` → `'2026-...Z'`).

So ~35% of the diff is churn in fields the estimator has no business editing. Across 125
actionable tasks that would be roughly a thousand lines of unrelated reflow. **Decision:
do not bulk write.** The ranking above came from `--dry-run` JSON, which is
non-destructive and gave the identical numbers — the writes were never needed to answer
the question. This is the same finding already filed as **T-2222** ("BVP estimator
corrupts anchor-less task frontmatter", itself scoring 72), so it is a known, independently
observed defect rather than a one-off.

### 2026-08-17 — the churn probe accidentally proved the `_cost_short_rationale` bug in situ
Unplanned, and stronger evidence than the code read. T-2644's generated rationale was:

    D1=4 (body:structural-gate); D2=3 (body:component-silent-failure);
    D3=2 (body:default-change); D4=2 (body:env-class-handled);
    F-RECALL=0 (no-signal); F-ORCH=1 (body:hand-wired-dispatch)

Five drivers render their signal; one renders `(no-signal)` — **same renderer, same task,
same run**. The only difference is evidence shape: F-RECALL scored 0 and returned a
single-element list, exactly like every cost scorer. That is the mechanism demonstrated
on live data rather than inferred from reading the source.

It also sharpens the bug's scope, which the filed report already had right: for VALUE,
`(no-signal)` is *truthful* when a driver genuinely found no body marker. For COST it is
*false*, because the cost scorers compute a signal and then bury it inside the arrow where
the renderer cannot see it. The defect is not "the renderer says no-signal" — it is "the
renderer cannot distinguish absent signal from unrecognised evidence shape, and asserts
the former."

Filed to `framework:pickup` at **offset 5** (2026-08-17), not patched: `estimator.py`
lives under gitignored `.agentic-framework/` — cross-repo, human governance (G-062).


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
