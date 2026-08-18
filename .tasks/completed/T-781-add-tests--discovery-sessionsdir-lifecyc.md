---
id: T-781
name: "Add tests — discovery sessions_dir, lifecycle serde edge cases, liveness cleanup
  TCP"
description: >
  Add tests — discovery sessions_dir, lifecycle serde edge cases, liveness cleanup
  TCP

status: work-completed
workflow_type: test
owner: agent
horizon:
tags: []
components: [crates/termlink-session/src/discovery.rs, 
      crates/termlink-session/src/lifecycle.rs]
related_tasks: []
created: 2026-03-30T00:18:04Z
last_update: '2026-08-18T18:59:21Z'
date_finished: 2026-03-30T00:19:31Z
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
      tier: 1
      effort: 3
    rationale: blast_radius=3 (no-signal); tier=1 (no-signal); effort=3 
      (no-signal)
    rubric_sha: missing
---

# T-781: Add tests — discovery sessions_dir, lifecycle serde edge cases, liveness cleanup TCP

## Context

discovery.rs has 2 tests, lifecycle.rs has 7 tests. Adding edge cases for sessions_dir path resolution and lifecycle serde/Display for all states.

## Acceptance Criteria

### Agent
- [x] discovery.rs: sessions_dir is a child of runtime_dir test
- [x] lifecycle.rs: serde roundtrip for all 5 states, Display for all 5, initializing rejects all
- [x] All new tests pass

## Verification

grep -q "fn sessions_dir_is_child_of_runtime" /opt/termlink/crates/termlink-session/src/discovery.rs
grep -q "fn all_states_serde_roundtrip" /opt/termlink/crates/termlink-session/src/lifecycle.rs

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

### 2026-03-30T00:18:04Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-781-add-tests--discovery-sessionsdir-lifecyc.md
- **Context:** Initial task creation

### 2026-03-30T00:19:31Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
