---
id: T-3033
name: "Session-daemon kv.*/session.* invocation blind spot (C-30/C-31) remains unmeasured"
description: >
  T-2996 instruments the MCP tool surface only. The session-daemon kv.* and session.*
  surfaces (value-review findings C-30/C-31) reach neither rpc_audit's authenticated-dispatch
  path nor the new invocation_audit choke point, so their usage is structurally unobservable.
  Owed as a follow-up by T-2996 AC 8. Scope this before assuming the MCP instrument
  generalises: the daemon surface may need its own choke point rather than a reuse
  of the MCP one.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [value-review, arc:arc-009]
components: [crates/termlink-session/src/audit_append.rs, crates/termlink-hub/src/rpc_audit.rs, crates/termlink-session/src/invocation_audit.rs, 
      crates/termlink-session/src/handler.rs, crates/termlink-hub/src/lib.rs]
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
created: 2026-09-21T07:32:16Z
last_update: 2026-09-29T10:08:31Z
date_finished: 2026-09-29T10:08:31Z
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
      D2: 0
      D3: 3
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=3 
      (body:component-discoverability); D4=2 (body:env-class-handled); 
      F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: e4a00f38e801
  - ts: '2026-09-29T10:04:46Z'
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
  - ts: '2026-09-22T14:57:41Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius:
      tier: 2
      effort: 8
    rationale: blast_radius=? (no-components-UNMEASURED-not-zero); tier=2 
      (workflow:build); effort=8 (lines=204,acs=4)
    rubric_sha: e4a00f38e801
  - ts: '2026-09-29T10:04:46Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 2
      effort: 6
    rationale: blast_radius=3 (3-components); tier=2 (workflow:build); effort=6 
      (lines=175,acs=3)
    rubric_sha: e4a00f38e801
---

# T-3033: Session-daemon kv.*/session.* invocation blind spot (C-30/C-31) remains unmeasured

## Context

Scoped 2026-09-29 (T-3211 R4), as the description asked, before assuming reuse.
- **Why the MCP instrument cannot simply be called:** `invocation_audit` lives in `termlink-hub`, and `termlink-hub` depends on `termlink-session`, not the reverse. The session daemon cannot call it.
- **Why a move, not a second recorder:** the module depends on `termlink_session::discovery::runtime_dir` (already in the session crate) and on hub's `rpc_audit::append_line_capped` + `rotated_path`, the shared O_APPEND/rotation writer. *(Correction during build: the first scoping missed that second dependency.)* Both helpers are pure std and moved to `termlink_session::audit_append`; hub's `rpc_audit` imports them, so there is still exactly one write path. Moving the file down and re-exporting it from hub (`pub use termlink_session::invocation_audit;`) keeps every existing call site (`termlink_hub::invocation_audit::…` in MCP and CLI) byte-identical. A second recorder would duplicate the O_APPEND/rotation logic whose correctness is the module's whole point (undercount → wrong deletion).
- **Choke point:** every session-daemon RPC enters through `handler::dispatch_scoped` (sole caller: `server.rs:78`).
- **Filter:** only `kv.*` and `session.*`, the surfaces C-30/C-31 name. Recording every method would include `event.poll`/`event.subscribe` long-poll loops, which are high-rate and would crowd the 32 MB sink without answering any open question.

## Acceptance Criteria

### Agent
- [x] `invocation_audit` lives in `termlink-session`; `termlink_hub::invocation_audit` still resolves (re-export) so MCP/CLI call sites are unchanged
- [x] `dispatch_scoped` records `kv.*` and `session.*` methods under a distinct `session-rpc` surface, and nothing else (unit-tested: kv.get recorded, event.poll not; live: a scratch session under a temp runtime dir recorded `session-rpc kv.set` / `kv.get` and nothing for its `list` queries)
- [x] The module's existing unit tests pass in their new crate; `cargo build -p termlink` and `cargo test -p termlink-session invocation` pass

## Verification

cargo test -p termlink-session --lib invocation > /tmp/.t3033-v1.out 2>&1 && grep -q "8 passed; 0 failed" /tmp/.t3033-v1.out
cargo test -p termlink-hub --lib rpc_audit > /tmp/.t3033-v2.out 2>&1 && grep -q "41 passed; 0 failed" /tmp/.t3033-v2.out
cargo build -p termlink -p termlink-mcp > /tmp/.t3033-v3.out 2>&1 && ! grep -q "^warning" /tmp/.t3033-v3.out
grep -q "pub use termlink_session::invocation_audit;" crates/termlink-hub/src/lib.rs
grep -q "is_recorded_session_method(&req.method)" crates/termlink-session/src/handler.rs

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

### 2026-09-29 — the move was two modules, not one
- **What changed:** scoping said invocation_audit had one non-std dependency; the compiler found a second (the shared capped-append writer in hub's rpc_audit).
- **Plan impact:** moved that writer too, into `termlink_session::audit_append`, rather than duplicating it. The module's own rationale (undercount leads to a wrong deletion) rules out a copy.
- **Triggered:** none. `session.*` methods served by the HUB (e.g. `session.discover`) are still counted by rpc_audit, not here; this covers the daemon side only.

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

### 2026-09-21T07:32:16Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3033-session-daemon-kvsession-invocation-blin.md
- **Context:** Initial task creation

### 2026-09-29T10:04:58Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

## Reviewer Verdict (v1.5)

- **Scan ID:** R-1b036537
- **Timestamp:** 2026-09-29T10:08:33Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Reviewer:** inline
- **Findings:** none

### 2026-09-29T10:08:31Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
