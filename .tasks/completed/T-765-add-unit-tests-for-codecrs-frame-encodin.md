---
id: T-765
name: "Add unit tests for codec.rs frame encoding/decoding edge cases"
description: >
  Add unit tests for codec.rs frame encoding/decoding edge cases

status: work-completed
workflow_type: test
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-29T22:53:59Z
last_update: 2026-03-29T22:58:15Z
date_finished: 2026-03-29T22:56:05Z
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
