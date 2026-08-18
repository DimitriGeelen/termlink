---
id: T-765
name: "Add unit tests for codec.rs frame encoding/decoding edge cases"
description: >
  Add unit tests for codec.rs frame encoding/decoding edge cases

status: work-completed
workflow_type: test
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-29T22:53:59Z
last_update: '2026-08-18T18:59:20Z'
date_finished: 2026-03-29T22:56:05Z
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
      tier: 1
      effort: 6
    rationale: blast_radius=0 (no-signal); tier=1 (no-signal); effort=6 
      (no-signal)
    rubric_sha: missing
---

# T-765: Add unit tests for codec.rs frame encoding/decoding edge cases

## Context

`crates/termlink-session/src/codec.rs` has 4 existing tests covering basic roundtrip, multi-frame, sequence increment, and large payload. Edge cases like empty payload, raw frame writes, all frame types, interleaved channels, and combined flags are not tested.

## Acceptance Criteria

### Agent
- [x] Add test for empty payload frame roundtrip through codec
- [x] Add test for `write_raw_frame` preserving sequence (not auto-incrementing)
- [x] Add test for all 8 frame types through async codec reader/writer
- [x] Add test for interleaved multi-channel frames
- [x] Add test for combined flags (FIN+COMPRESSED+BINARY+URGENT) roundtrip
- [x] All new tests pass via `cargo test -p termlink-session codec`

## Verification

cargo test -p termlink-session codec

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

### 2026-03-29T22:53:59Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-765-add-unit-tests-for-codecrs-frame-encodin.md
- **Context:** Initial task creation

### 2026-03-29T22:56:05Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
