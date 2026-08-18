---
id: T-542
name: "Enrich termlink info with version and build details"
description: >
  Enrich termlink info with version and build details

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-cli/src/commands/session.rs]
related_tasks: []
created: 2026-03-28T09:18:46Z
last_update: '2026-08-18T18:59:17Z'
date_finished: 2026-03-28T09:19:48Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:01Z'
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
  - ts: '2026-08-18T18:59:17Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 2
      effort: 3
    rationale: blast_radius=1 (no-signal); tier=2 (no-signal); effort=3 
      (no-signal)
    rubric_sha: missing
---

# T-542: Enrich termlink info with version and build details

## Context

`termlink info` shows runtime paths and session counts but not version, commit, or build target. Add build info section for diagnostic completeness.

## Acceptance Criteria

### Agent
- [x] `termlink info` includes version, commit, and build target
- [x] `termlink info --json` includes version info in JSON
- [x] `cargo build` succeeds

## Verification

cargo build 2>&1
./target/debug/termlink info 2>&1 | grep -q "Version:"

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

### 2026-03-28T09:18:46Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-542-enrich-termlink-info-with-version-and-bu.md
- **Context:** Initial task creation

### 2026-03-28T09:19:48Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
