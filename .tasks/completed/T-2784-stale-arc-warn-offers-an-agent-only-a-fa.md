---
id: T-2784
name: "Stale-arc WARN offers an agent only a falsification remedy (AEF audit.sh)"
description: >
  Stale-arc WARN offers an agent only a falsification remedy (AEF audit.sh)

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
created: 2026-08-17T14:38:35Z
last_update: '2026-08-18T18:59:16Z'
date_finished: 2026-08-17T14:44:37Z
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
      D2: 5
      D3: 4
      D4: 2
      F-RECALL: 2
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=5 (body:silent-class-removed); 
      D3=4 (body:framework-level-ux); D4=2 (body:env-class-handled); F-RECALL=2 
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

# T-2784: Stale-arc WARN offers an agent only a falsification remedy (AEF audit.sh)

## Context

The `fw audit` structure section emits `Arc '<slug>' has no task commits in the last 30
days` and prints two remedies. For an **autonomous agent**, neither is available:

1. `fw arc close` — sovereignty-gated. `lib/arc.sh:671` refuses under `$CLAUDECODE=1`
   unless `--i-am-human` / `--from-watchtower`. An agent structurally cannot.
2. `fw task update T-XXX --last-update <ts>` — **the flag does not exist**.
   `update-task.sh:1323` falls through to `*) Unknown option: $1; exit 1`.

So a WARN that fires at an agent has, for that agent, **zero actionable remedies**. It has
fired **14 times across 5 days** (`.context/audits/2026-08-12..16.yaml`) on
`arc-substrate-fitness` alone.

Even if remedy 2 worked, it prescribes touching an activity timestamp to silence a
staleness check *without doing any work* — the framework instructing its operator to
falsify the record a guard reads. That is worse than a noisy guard: it is a guard whose
documented fix is dishonesty.

The underlying detector explains why the arc is flagged at all. `audit.sh:899` decides
staleness purely by `git log --since=<N>.days.ago -- <task files>`. It never reads task
`status` or `horizon`. `arc-substrate-fitness` is **11/12 `work-completed`**; the twelfth
(T-2250, R5 telemetry design) is `captured` at `horizon: later` — parked by an explicit
human decision recorded in the arc's own resolved-decisions block ("R5(telemetry
inception)" last in the surviving order). A **finished-but-parked** arc is therefore
indistinguishable from a **stalled** one, and only the latter is what the check claims to
detect.

**Scope.** `audit.sh`, `arc.sh` and `update-task.sh` all live under `.agentic-framework/`
— gitignored, cross-repo, human governance (G-062). This task does **not** fix them. The
deliverable is an evidenced finding filed to AEF via `framework:pickup` (the established
path, used twice before this session), plus surfacing the one human-only action.

## Acceptance Criteria

### Agent
- [x] Report artifact `docs/reports/T-2784-stale-arc-warn-unactionable.md` exists and
      states all four claims with file:line citations
- [x] Claim 1 cited: `arc_close` refuses under `$CLAUDECODE=1` without `--i-am-human`
      (`.agentic-framework/lib/arc.sh:671`)
- [x] Claim 2 reproduced empirically: `fw task update T-2250 --last-update <ts>` exits
      non-zero with `Unknown option`, verbatim output recorded in the report
- [x] Claim 3 cited: the detector reads git-commit recency only, never `status`/`horizon`
      (`.agentic-framework/agents/audit/audit.sh:899`)
- [x] Claim 4 measured: arc-002 task-status census (11 `work-completed` / 1 `captured`
      `horizon: later`) and the 14-fire-in-5-days count both recorded in the report
- [x] Finding filed to the `framework:pickup` hub topic, carrying the four claims and a
      proposed remedy, and the post offset recorded in the report
- [x] The human-only action (closing arc-002) surfaced to the user as a single
      copy-pasteable `cd … && .agentic-framework/bin/fw …` line per T-609

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

# Report artifact exists and carries all four claims with citations.
test -f docs/reports/T-2784-stale-arc-warn-unactionable.md
out=$(cat docs/reports/T-2784-stale-arc-warn-unactionable.md); grep -q "arc.sh:671" <<< "$out"
out=$(cat docs/reports/T-2784-stale-arc-warn-unactionable.md); grep -q "update-task.sh:1323" <<< "$out"
out=$(cat docs/reports/T-2784-stale-arc-warn-unactionable.md); grep -q "audit.sh:899" <<< "$out"
out=$(cat docs/reports/T-2784-stale-arc-warn-unactionable.md); grep -q "Unknown option: --last-update" <<< "$out"
out=$(cat docs/reports/T-2784-stale-arc-warn-unactionable.md); grep -q "framework:pickup" <<< "$out"
# Claim 1 still true in the live AEF tree (gate present).
out=$(sed -n '671p' .agentic-framework/lib/arc.sh); grep -q 'CLAUDECODE' <<< "$out"
# Claim 2 still true: --last-update is rejected. Non-zero exit is the PASS here.
out=$(.agentic-framework/bin/fw task update T-2250 --last-update 2026-01-01T00:00:00Z 2>&1 || true); grep -q "Unknown option" <<< "$out"
# Claim 3 still true: the detector's staleness decision reads git log, not task status.
out=$(sed -n '899p' .agentic-framework/agents/audit/audit.sh); grep -q 'git .*log .*--since' <<< "$out"

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

### 2026-08-17T14:38:35Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2784-stale-arc-warn-offers-an-agent-only-a-fa.md
- **Context:** Initial task creation

### 2026-08-17T14:44:37Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
