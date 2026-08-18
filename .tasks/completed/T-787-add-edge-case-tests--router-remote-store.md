---
id: T-787
name: "Add edge case tests — router remote store uninit, pidfile parsing, orchestrator
  empty candidates"
description: >
  Add edge case tests — router remote store uninit, pidfile parsing, orchestrator
  empty candidates

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-hub/src/pidfile.rs, 
      crates/termlink-hub/src/remote_store.rs]
related_tasks: []
created: 2026-03-30T12:24:16Z
last_update: '2026-08-18T18:59:21Z'
date_finished: 2026-03-30T12:28:42Z
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
  - ts: '2026-08-18T18:59:21Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 2
      effort: 3
    rationale: blast_radius=3 (no-signal); tier=2 (no-signal); effort=3 
      (no-signal)
    rubric_sha: missing
---

# T-787: Add edge case tests — router remote store uninit, pidfile parsing, orchestrator empty candidates

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] Pidfile edge case tests added (empty, whitespace, trailing newline, overflow, negative, error Display, std::error::Error)
- [x] Remote store reaper tests added (expired removal, shutdown signal)
- [x] All workspace tests pass (629)

## Verification

cargo test -p termlink-hub --lib 2>&1 | grep "0 failed"

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

### 2026-03-30T12:24:16Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-787-add-edge-case-tests--router-remote-store.md
- **Context:** Initial task creation

### 2026-03-30T12:28:42Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
