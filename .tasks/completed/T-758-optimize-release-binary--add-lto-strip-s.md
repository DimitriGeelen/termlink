---
id: T-758
name: "Optimize release binary — add LTO, strip symbols, single codegen unit"
description: >
  Optimize release binary — add LTO, strip symbols, single codegen unit

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-29T19:54:54Z
last_update: '2026-08-18T18:59:20Z'
date_finished: 2026-03-29T19:59:20Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:09Z'
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
  - ts: '2026-08-18T18:59:20Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 4
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=4 
      (no-signal)
    rubric_sha: missing
---

# T-758: Optimize release binary — add LTO, strip symbols, single codegen unit

## Context

Release binary is 18MB with default profile. Adding LTO, stripping, and single codegen unit should reduce size significantly.

## Acceptance Criteria

### Agent
- [x] `[profile.release]` section added to workspace Cargo.toml with lto, strip, codegen-units
- [x] Release binary builds successfully
- [x] Release binary is 12MB (down from 18MB — 33% reduction)
- [x] All 528 tests pass

## Verification

grep -q "profile.release" Cargo.toml
test -f target/release/termlink

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

### 2026-03-29T19:54:54Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-758-optimize-release-binary--add-lto-strip-s.md
- **Context:** Initial task creation

### 2026-03-29T19:59:20Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
