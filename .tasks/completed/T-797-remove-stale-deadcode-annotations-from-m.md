---
id: T-797
name: "Remove stale dead_code annotations from manifest.rs"
description: >
  Remove dead_code allow annotations now that methods are used

status: work-completed
workflow_type: refactor
owner: agent
horizon:
tags: []
components: [crates/termlink-cli/src/manifest.rs]
related_tasks: []
created: 2026-03-30T14:23:04Z
last_update: '2026-08-18T18:59:21Z'
date_finished: 2026-03-30T14:35:26Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:10Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=0 (no-signal); D4=0 
      (no-signal); F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:21Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 3
      effort: 2
    rationale: blast_radius=1 (no-signal); tier=3 (no-signal); effort=2 
      (no-signal)
    rubric_sha: missing
---

# T-797: Remove stale dead_code annotations from manifest.rs

## Context

`pending_dispatches()` and `count_by_status()` in manifest.rs had `#[allow(dead_code)]` added when they were created (T-791) before their caller `cmd_dispatch_status` (T-793) existed. Now both are actively used — annotations are stale.

## Acceptance Criteria

### Agent
- [x] No `#[allow(dead_code)]` annotations remain in manifest.rs
- [x] `cargo check -p termlink 2>&1` produces no warnings (clean compile)

## Verification

grep -qv 'allow(dead_code)' crates/termlink-cli/src/manifest.rs || ! grep -q 'allow(dead_code)' crates/termlink-cli/src/manifest.rs

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

### 2026-03-30T14:23:04Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-797-remove-stale-deadcode-annotations-from-m.md
- **Context:** Initial task creation

### 2026-03-30T14:33:48Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-30T14:35:26Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
