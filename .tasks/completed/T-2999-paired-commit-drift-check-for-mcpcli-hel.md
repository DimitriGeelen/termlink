---
id: T-2999
name: "Paired-commit drift check for _mcp/CLI helper twins"
description: >
  S-25/C-26 alt half: T-2069 duplicates tiny pure helpers across CLI and MCP crates
  by convention; add a paired-commit drift check that flags a commit touching one
  twin without the other (no change to the T-2069 convention itself). Evidence: docs/reports/VALUE-REVIEW-repo-2026-09-19-consolidated.md
  C-26.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [value-review, arc:arc-009]
components: [scripts/check-mcp-cli-twin-drift.sh, 
      tests/mcp-cli-twin-drift-fixtures.sh]
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
created: 2026-09-19T22:22:09Z
last_update: 2026-09-29T10:18:56Z
date_finished: 2026-09-29T10:18:56Z
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
  - ts: '2026-09-20T08:45:11Z'
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
  - ts: '2026-09-29T10:13:58Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 3
      D4: 3
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=3 
      (body:component-discoverability); D4=3 (body:portability-abstraction); 
      F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: e4a00f38e801
cost_estimate_proposed:
  - ts: '2026-09-20T08:45:20Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius:
      tier: 2
      effort: 8
    rationale: blast_radius=? (no-components-UNMEASURED-not-zero); tier=2 
      (workflow:build); effort=8 (lines=207,acs=4)
    rubric_sha: e4a00f38e801
  - ts: '2026-09-29T10:13:58Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 2
      effort: 6
    rationale: blast_radius=3 (2-components); tier=2 (workflow:build); effort=6 
      (lines=176,acs=3)
    rubric_sha: e4a00f38e801
---

# T-2999: Paired-commit drift check for _mcp/CLI helper twins

## Context

C-26: 67 `fn *_mcp` helpers in `crates/termlink-mcp/src` (63 with a same-named CLI twin in `crates/termlink-cli/src`, measured 2026-09-29) are duplicated by the T-2069 convention, and duplication has already produced undetected parity drift (45 days, per the finding; T-3214 is a live instance). The finding's no-sovereignty option: "add a check that both twins changed in the same commit". Extracting shared fns would revisit T-2069 and is a sovereign question, so it is out of scope.

Design: `scripts/check-mcp-cli-twin-drift.sh [--range A..B] [--json]`. For each commit in the range, map every changed line (both sides, `-U0`) to its nearest preceding `fn` in that file version, then flag each twin pair where exactly one side was touched. It is a **review list, not a gate**: a CLI-only presentation change is legitimate, so a hit means "look", not "wrong". It is also range-based, so its verdict depends on the history you point it at, and for that reason it is deliberately **not** a `# guard-layer: source` member. Exit 0 = no one-sided change, 1 = one-sided change(s) listed, 2 = tooling (bad range, not a git repo; never a vacuous clean).

**Measured on this repo (2026-09-29):** `--range HEAD~3000..HEAD` scanned 3230 commits and 67 twin pairs in 75s → **29 one-sided modifications of established pairs** (49 before excluding creations). The first one spot-checked is **real, live drift**: T-2619 (`8154379b1`) made the CLI `parse_find_idle_log` / `parse_substrate_log` / `parse_queue_log` count an unparseable `ts` as malformed; the MCP twins still pass `ts` through `fleet_history_rfc3339_to_unix`, which returns a `0` sentinel (`tools.rs:256-258`), so a bad-ts row is silently dropped by the cutoff and never counted. Filed separately (one lock at a time).

## Acceptance Criteria

### Agent
- [x] Script flags a commit that changes only `foo_mcp` (or only `foo`) and stays quiet when both change in the same commit, or when neither is a twin (hermetic fixture repo)
- [x] Bad range / non-repo exits 2, never 0
- [x] Run over this repo's recent history, with the count reported in Context as evidence of the signal's size

## Verification

bash tests/mcp-cli-twin-drift-fixtures.sh > /tmp/.t2999-v1.out 2>&1 && grep -q "7 passed, 0 failed" /tmp/.t2999-v1.out
bash -n scripts/check-mcp-cli-twin-drift.sh
bash scripts/check-mcp-cli-twin-drift.sh --range HEAD~100..HEAD --json > /tmp/.t2999-v3.out 2>&1; test $? -le 1 && python3 -c "import json;d=json.load(open('/tmp/.t2999-v3.out'));assert d['twin_pairs']>=60 and d['commits_scanned']>=100"

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

### 2026-09-29 — creation is not drift
- **What changed:** the first version flagged 49 commits in 3230, most of them the normal "CLI first, MCP parity next commit" creation sequence.
- **Plan impact:** a hit now requires an established pair (the changed twin existed in the parent, the other exists at the commit). 29 remain, and the first one read is real drift.
- **Triggered:** a bug task for the T-2619 MCP-twin drift (history parsers).

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

### 2026-09-19T22:22:09Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2999-paired-commit-drift-check-for-mcpcli-hel.md
- **Context:** Initial task creation

### 2026-09-19T22:35:33Z — status-update [task-update-agent]
- **Change:** tags: +arc:arc-009

### 2026-09-29T10:14:09Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

## Reviewer Verdict (v1.5)

- **Scan ID:** R-90670443
- **Timestamp:** 2026-09-29T10:19:00Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Reviewer:** inline
- **Findings:** none

### 2026-09-29T10:18:56Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
