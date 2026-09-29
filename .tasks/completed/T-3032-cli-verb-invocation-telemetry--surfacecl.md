---
id: T-3032
name: "CLI verb invocation telemetry — SURFACE_CLI exists but nothing calls it"
description: >
  T-2996 shipped invocation_audit with SURFACE_CLI defined and zero call sites: the
  MCP tool surface is instrumented, the CLI verb surface is not. Deliberately excluded
  from T-2996 scope and owed as a follow-up by its AC 8. Until this lands, invocation-usage.sh
  can only answer 'not observed on the MCP surface', never 'unused' — and C-45's usage
  question spans both surfaces. The choke point is the CLI dispatch site; the sink,
  rotation and concurrency guarantees already exist and need no redesign.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [value-review, arc:arc-009]
components: [crates/termlink-cli/src/main.rs, scripts/invocation-usage.sh]
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
created: 2026-09-21T07:31:14Z
last_update: 2026-09-29T10:03:48Z
date_finished: 2026-09-29T10:03:48Z
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
  - ts: '2026-09-22T14:57:18Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 2
      D3: 3
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=2 
      (body:telemetry-or-audit-entry); D3=3 (body:component-discoverability); 
      D4=2 (body:env-class-handled); F-RECALL=0 (no-signal); F-ORCH=0 
      (no-signal)
    rubric_sha: e4a00f38e801
  - ts: '2026-09-29T10:00:22Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 2
      D3: 3
      D4: 3
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=2 
      (body:telemetry-or-audit-entry); D3=3 (body:component-discoverability); 
      D4=3 (body:portability-abstraction); F-RECALL=0 (no-signal); F-ORCH=0 
      (no-signal)
    rubric_sha: e4a00f38e801
cost_estimate_proposed:
  - ts: '2026-09-22T14:57:41Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius:
      tier: 2
      effort: 8
    rationale: blast_radius=? (no-components-UNMEASURED-not-zero); tier=2 
      (workflow:build); effort=8 (lines=204,acs=4)
    rubric_sha: e4a00f38e801
  - ts: '2026-09-29T10:00:23Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 2
      effort: 7
    rationale: blast_radius=3 (2-components); tier=2 (workflow:build); effort=7 
      (lines=174,acs=4)
    rubric_sha: e4a00f38e801
---

# T-3032: CLI verb invocation telemetry — SURFACE_CLI exists but nothing calls it

## Context

T-2996 (C-45) built `termlink_hub::invocation_audit` with `SURFACE_MCP` wired at `termlink-mcp/src/server.rs::call_tool` and `SURFACE_CLI` defined with **zero call sites** (`grep -rn SURFACE_CLI crates/*/src` → only the const). So `scripts/invocation-usage.sh` states its scope as "MCP tool surface only" and every CLI-verb usage question still reads UNMEASURED.

Design: record once in `main()`, after clap has parsed (so `--help`/`--version`/usage errors, which exit inside clap, are not counted as verb use), with the verb as the **subcommand-name chain** (`channel post`, `agent find-idle`). Only subcommand names are recorded, never argument values, so session names, messages and secrets cannot reach the sink. `Cli::parse()` becomes `Cli::command().get_matches()` + `Cli::from_arg_matches()`, which is clap's own decomposition of `parse()`, so parse and exit behaviour is unchanged. Recording is infallible by the module's contract.

## Acceptance Criteria

### Agent
- [x] Every CLI invocation that parses successfully appends one `surface:"cli"` record whose name is the subcommand chain (unit-tested for top-level, nested, and "positional values never recorded")
- [x] A real `termlink` run writes the record to the sink in `TERMLINK_RUNTIME_DIR`; `TERMLINK_INVOCATION_AUDIT=0` writes nothing
- [x] `scripts/invocation-usage.sh` scope text no longer claims CLI verbs are uninstrumented, and it reports a CLI record
- [x] `cargo build -p termlink` and the CLI crate's unit tests pass

## Verification

cargo test -p termlink --bin termlink cli_verb_path > /tmp/.t3032-v1.out 2>&1 && grep -q "3 passed; 0 failed" /tmp/.t3032-v1.out
cargo build -p termlink --quiet
R=$(mktemp -d) && TERMLINK_RUNTIME_DIR=$R target/debug/termlink channel list --json >/dev/null 2>&1; grep -q '"surface":"cli","name":"channel list"' $R/invocation-audit.jsonl
R=$(mktemp -d) && TERMLINK_INVOCATION_AUDIT=0 TERMLINK_RUNTIME_DIR=$R target/debug/termlink list >/dev/null 2>&1; test ! -e $R/invocation-audit.jsonl
bash scripts/invocation-usage.sh --help > /tmp/.t3032-v5.out 2>&1 && grep -q "per-verb CLI" /tmp/.t3032-v5.out

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

### 2026-09-29 — one call site, and the reader's scope text
- **What changed:** the sink, rotation and O_APPEND safety all existed (T-2996); the CLI needed one call site. The half that was easy to miss was the reader: `invocation-usage.sh` stated "CLI verbs are NOT instrumented" in both its header and its runtime SCOPE line, and would have gone on saying so while recording them.
- **Plan impact:** none. Recording is placed after clap's parse, so `--help`/usage errors are not counted as verb use.
- **Triggered:** none. CLI subprocesses spawned by MCP tools (e.g. `fleet bootstrap-check`) will also record as `cli`. That is correct (the verb ran), but counts should be read with it in mind. The kv.*/session.* surface remains T-3033.

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

### 2026-09-21T07:31:14Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3032-cli-verb-invocation-telemetry--surfacecl.md
- **Context:** Initial task creation

### 2026-09-29T10:00:44Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

## Reviewer Verdict (v1.5)

- **Scan ID:** R-345442df
- **Timestamp:** 2026-09-29T10:03:50Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Reviewer:** inline
- **Findings:** none

### 2026-09-29T10:03:48Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
