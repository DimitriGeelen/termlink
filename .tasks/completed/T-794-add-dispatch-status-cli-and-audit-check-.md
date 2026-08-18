---
id: T-794
name: "Add dispatch status CLI and audit check for orphaned branches"
description: >
  Phase 5: dispatch status subcommand + audit section for orphaned branches

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: [T-789, T-793]
created: 2026-03-30T13:35:20Z
last_update: '2026-08-18T18:59:21Z'
date_finished: 2026-03-30T14:13:00Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:10Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 3
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 1
    rationale: D1=3 (body:test-or-audit-check); D2=0 (no-signal); D3=0 
      (no-signal); D4=0 (no-signal); F-RECALL=0 (no-signal); F-ORCH=1 
      (body:hand-wired-dispatch)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:21Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 4
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=4 
      (no-signal)
    rubric_sha: missing
---

# T-794: Add dispatch status CLI and audit check for orphaned branches

## Context

Phase 5 of T-789. The CLI status command was delivered in T-793 (`termlink dispatch-status`). This task updates ARCHITECTURE.md and CHANGELOG.md with the new worktree isolation feature, new commands, and updated test counts.

## Acceptance Criteria

### Agent
- [x] ARCHITECTURE.md updated with new test count
- [x] CHANGELOG.md updated with worktree isolation features (--isolate, --auto-merge, --workdir, dispatch-status)
- [x] `dispatch-status` command documented
- [x] New manifest module documented

## Verification

grep -q "dispatch-status" CHANGELOG.md
grep -q "isolate" CHANGELOG.md

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

### 2026-03-30T13:35:20Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-794-add-dispatch-status-cli-and-audit-check-.md
- **Context:** Initial task creation

### 2026-03-30T14:11:19Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-30T14:13:00Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
