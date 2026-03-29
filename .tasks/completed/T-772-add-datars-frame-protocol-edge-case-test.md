---
id: T-772
name: "Add data.rs frame protocol edge case tests"
description: >
  Add data.rs frame protocol edge case tests

status: work-completed
workflow_type: test
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-29T23:29:49Z
last_update: 2026-03-29T23:31:19Z
date_finished: 2026-03-29T23:31:19Z
---

# T-772: Add data.rs frame protocol edge case tests

## Context

data.rs has 6 tests covering roundtrip, magic, flags, all types. Missing: invalid type rejection, incomplete buffer, unsupported version, truncated payload, max value boundaries.

## Acceptance Criteria

### Agent
- [x] Add test for FrameType::from_u8 with invalid values (0x08, 0xFF)
- [x] Add test for FrameHeader::decode with incomplete buffer (too short, empty)
- [x] Add test for unsupported protocol version rejection
- [x] Add test for Frame::decode with truncated payload
- [x] Add test for zero-payload frame roundtrip
- [x] Add test for max sequence/channel_id boundary values
- [x] Add test for version field preservation in header
- [x] All tests pass: `cargo test -p termlink-protocol` (78 tests, 0 failures)

## Verification

cargo test -p termlink-protocol -- --quiet

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

### 2026-03-29T23:29:49Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-772-add-datars-frame-protocol-edge-case-test.md
- **Context:** Initial task creation

### 2026-03-29T23:31:19Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
