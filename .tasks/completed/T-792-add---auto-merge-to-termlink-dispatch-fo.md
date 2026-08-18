---
id: T-792
name: "Add --auto-merge to termlink dispatch for worktree branch reconciliation"
description: >
  Phase 3: sequential merge of dispatch worker branches back to base

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-cli/src/cli.rs, 
      crates/termlink-cli/src/commands/dispatch.rs, 
      crates/termlink-cli/src/main.rs, crates/termlink-cli/src/manifest.rs]
related_tasks: [T-789, T-791, T-793]
created: 2026-03-30T13:35:14Z
last_update: '2026-08-18T18:59:21Z'
date_finished: 2026-03-30T14:05:23Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:10Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 2
      F-ORCH: 1
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=0 (no-signal); D4=0 
      (no-signal); F-RECALL=2 (body:lightly-promoted); F-ORCH=1 
      (body:hand-wired-dispatch)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:21Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 5
      tier: 2
      effort: 8
    rationale: blast_radius=5 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-792: Add --auto-merge to termlink dispatch for worktree branch reconciliation

## Context

Phase 3 of T-789 (worktree isolation). Adds `--auto-merge` flag that sequentially merges dispatch worker branches back to the base branch after collection. Updates manifest status to `merged` or `conflict`. See `docs/reports/T-789-worktree-isolation-research.md` merge orchestration section.

## Acceptance Criteria

### Agent
- [x] `--auto-merge` flag added to Dispatch CLI struct
- [x] After collection+cleanup, branches with commits are merged sequentially into base branch
- [x] Successful merges update manifest entry status to `merged`
- [x] Failed merges (conflict) update manifest status to `conflict` with reason
- [x] Conflicting branch is preserved (not deleted) for manual resolution
- [x] Merge order is deterministic (worker-1, worker-2, ...)
- [x] JSON output includes merge results per branch
- [x] `merge_branch()` function in manifest.rs handles git merge + fast-forward
- [x] Unit test: auto_merge requires isolate validation
- [x] All existing tests pass (649 total)
- [x] `cargo clippy --workspace` has zero warnings

## Verification

grep -q "auto.merge" crates/termlink-cli/src/cli.rs
grep -q "merge_branch" crates/termlink-cli/src/manifest.rs
grep -q "Merged" crates/termlink-cli/src/manifest.rs

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Updates

### 2026-03-30T13:35:14Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-792-add---auto-merge-to-termlink-dispatch-fo.md
- **Context:** Initial task creation

### 2026-03-30T13:59:36Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-30T14:05:23Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
