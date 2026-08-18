---
id: T-2776
name: "Work HV/LC + HV/HC BVP-ranked stabilisation items"
description: >
  Rank actionable tasks by BVP quadrant (hv-lc then hv-hc), pick the highest-value
  items that stabilise termlink, and execute them under framework governance.

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-16T20:27:06Z
last_update: '2026-08-18T18:59:16Z'
date_finished: 2026-08-16T20:52:44Z
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
  - ts: '2026-08-18T18:57:00Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 2
      D4: 2
      F-RECALL: 2
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=2 
      (body:default-change); D4=2 (body:env-class-handled); F-RECALL=2 
      (body:lightly-promoted); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:16Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 8
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2776: Work HV/LC + HV/HC BVP-ranked stabilisation items

## Context

Triage task. The operator asked for work on HV/HC and HV/LC items that stabilise
termlink. The repo already has BVP scoring infrastructure (`fw bvp --quadrant`,
`bvp_scores`, `cost_estimate` — arc-006, semantics in
`docs/reports/T-1915-bvp-inception.md`), so the ranking is measured rather than
guessed.

This task's deliverable is the **selection**, not the implementation: run the
quadrant ranking, check inbound peer messages, and file a separate build task per
selected item (task-sizing rule: one task = one deliverable). Execution happens
under those task IDs.

## Acceptance Criteria

### Agent
- [x] `fw bvp --quadrant hv-lc` and `--quadrant hv-hc` both run and their rankings are recorded in `## Decisions` below (top items, with BVP and cost) — **both ran and returned no rows**; the reason is recorded in full rather than a ranking, because none exists to record
- [x] Inbound `framework:pickup` is read and any message newer than the last-acked offset is either actioned or explicitly triaged in `## Decisions`
- [x] Each selected item has its own build task filed with real (non-placeholder) ACs — T-2777; the Pen reply was a message not a build unit, the BVP finding is cross-repo (deferred with reason)
- [x] Selected items are executed to completion under their own task IDs, or explicitly deferred here with a reason
- [x] `bash scripts/run-guard-layer.sh` passes at the end of the session (no guard regressed)

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
       `bin/fw reviewer T-XXX 2>&1 | grep -q "Overall:.*PASS"` added to ## Verification.
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
# Pipefail/SIGPIPE hint (L-387): P-011 runs each command under `set -eo pipefail`.
# `cmd | grep -q PATTERN` exits 141 (SIGPIPE) when grep matches and closes stdin
# while the upstream is still writing — verification then "fails" even though
# the pattern was present. Safe pattern: capture first, grep the capture:
#     out=$(cmd 2>&1); echo "$out" | grep -q "PATTERN"
# Or:
#     cmd > /tmp/.out 2>&1 && grep -q "PATTERN" /tmp/.out
# Origin: L-387, captured 4× (T-1716, T-1838, T-1862, T-1863) before this hint.
#
# Single pipe only — no intermediate tail/awk/sed stages between capture and grep
# (T-2090): `echo "$out" | tail -3 | grep -q PAT` re-introduces the SIGPIPE risk
# the capture step closed off — the middle stage is what `grep -q` slams its
# stdin on. `echo "$out"` is small and immediate; grep scans the whole captured
# string anyway, so the tail-3 was cosmetic. Drop it: `echo "$out" | grep -q PAT`.
#
# Enforcement-baseline hint (L-398, T-1886): if you edited `.claude/settings.json`
# (added/removed/reorganised hooks), add `bin/fw enforcement baseline` to your
# Verification block. Otherwise the canonical hash diverges and `fw doctor`
# reports a FAIL ("Enforcement baseline CHANGED") that accumulates silently.
# Origin: T-1849/T-1730/T-1731 each added a legitimate hook without refreshing
# the baseline — FAIL sat for multiple sessions until T-1886 cleaned up.

out=$(bash scripts/run-guard-layer.sh 2>&1 || true); grep -q "guard layer: PASS" <<< "$out"
bash scripts/check-task-template-idioms.sh --no-heartbeat
bash scripts/check-verification-pipefail.sh
# the selected item actually landed
test -f .tasks/completed/T-2777-task-template-teaches-the-sigpipe-unsafe.md
# The 166 no-signal cost writes were reverted, not left behind to feed a bogus
# ranking: ZERO active tasks carry a cost_estimate_proposed: key. That zero is also
# the finding — cost has never been estimated for any task here, which is why
# `--quadrant` cannot place a single row. The 3 tasks that DO carry
# bvp_scores_proposed: (T-2022/24/26) have value without cost. Anchored with ^ so the
# template's commented-out `# cost_estimate_proposed:` line does not count — that
# miscount is what made 3 scored tasks look like 124 earlier in this task.
test "$(grep -l '^cost_estimate_proposed:' .tasks/active/*.md | wc -l)" -eq 0

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

### 2026-08-16 — the BVP quadrant ranking could not answer the question, and was not made to
- **Measured:** `fw bvp --quadrant hv-lc` and `--quadrant hv-hc` both return
  *"No tasks have `bvp_scores:` set yet."* Confirmed scores: **0 tasks**. Proposed
  scores: **3 of 224** active tasks (a `grep -l` suggesting 124 was matching the
  template's own commented-out `# bvp_scores_proposed:` line — the real count is 3).
  All three render `COST: -` and `QUAD: -`, so even those cannot be placed in a
  quadrant.
- **Root cause, and it is not a bug in `bvp.sh`:** the ranking needs BOTH value and
  cost. `fw bvp estimate` writes value only. Cost estimation exists and is complete —
  `cmd_cost_one` / `cmd_cost_all` / `cost-sweep` / `cost-determinism` in
  `agents/termlink/bvp-estimator/estimator.py` (T-1935) — but `fw bvp` has no `cost`
  dispatch and its `--help` never mentions one. The verbs are reachable only by
  invoking the estimator directly with `PROJECT_ROOT` set. So `cost_estimate` is
  absent everywhere, `quadrant()` returns None for every row, and `--quadrant` —
  a documented, advertised surface — can never return anything for any task.
  Measured directly: **zero** active tasks carry a `cost_estimate_proposed:` key, and
  zero carry a confirmed `cost_estimate:`. Cost has never been estimated for a single
  task in this project. Even the 3 tasks with proposed *value* scores have no cost, so
  they cannot be placed either — which is why `--include-proposed` lists them with
  `COST: -` and `QUAD: -`.
- **This is the T-2699 pattern:** coverage of a builder says nothing about whether the
  builder is called. Same shape as `check_protocol_version()` shipping with a passing
  test and zero callers.
- **Chose:** do NOT synthesise a ranking. I ran the cost estimator directly over the
  166 actionable tasks to see whether it would unblock the quadrant. It "succeeded" —
  and produced no-signal defaults: **all 166** carried `rationale: no-signal` on every
  component, 96 shared the identical `(blast_radius=0, tier=2, effort=8)` triple, 141
  had `blast_radius=0`, and every row recorded `rubric_sha: missing` because
  `policy/bvp-scoring-rubric.md` does not exist in this project (only
  `policy/value-drivers.yaml` does). Ranking on that would have produced a
  confident-looking, meaningless prioritisation — a plausible wrong answer, which is
  the Directive #2 failure shape, presented to the operator as measurement.
- **Rejected:** shipping the ranking anyway with a caveat. A caveat under a sorted
  table does not survive contact with a reader; the table is what gets acted on.
- **Reverted:** all 166 writes (`git checkout -- .tasks/active/`). They were not
  merely useless — with data present, `--include-proposed --quadrant hv-lc` would
  have started returning a bogus ranking that *looked* real. The estimator also
  reflowed every `description:` block and rewrote `date_finished: null` to empty,
  2951 insertions of churn for zero signal.
- **Fell back to:** judgement, stated as judgement. See the selection below.
- **Cross-repo:** `.agentic-framework/` is gitignored here (a vendored framework), so
  wiring the cost verbs into `fw bvp` and installing the scoring rubric are edits for
  its owner, not this repo (G-062). Filed rather than patched.

### 2026-08-16 — what was selected, and on what basis
Not BVP-ranked (see above). Ranked by: does it stop a defect from recurring, and is
it cheap?
1. **T-2777 — the task template taught the unsafe verification idiom.** Highest
   leverage available: it is the *source* of the class T-2775 had just spent a
   session ledgering, so fixing it stops new instances at the point of creation
   rather than cataloguing them afterwards. DONE — plus a guard so it stays fixed,
   and two corrections to T-2775's own check found along the way.
2. **Pen's request for the detector.** An explicit inbound ask on `framework:pickup`,
   answerable immediately. DONE — posted at offset 4, including a correction to the
   advice I had sent them the day before (`sh -c` does not propagate pipefail, so
   some lines they are about to ledger are not defects).
3. **The BVP wiring gap.** Recorded here; cross-repo, so not actioned.

### 2026-08-16 — inbound messages triaged
`framework:pickup` count=4. Offset 0 = AEF persistence probe (no action). Offset 1 =
our own T-2775 correction. Offset 2 = Pen's ack: they verified the 3M-line
measurement independently and updated their template in both places. Offset 3 = Pen
asking for the detector — actioned, offset 4. Nothing unprocessed.


<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-08-16T20:27:06Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2776-work-hvlc--hvhc-bvp-ranked-stabilisation.md
- **Context:** Initial task creation

### 2026-08-16T20:52:44Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
