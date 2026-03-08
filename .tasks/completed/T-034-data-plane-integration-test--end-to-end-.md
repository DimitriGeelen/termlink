---
id: T-034
name: "Data plane integration test — end-to-end register, stream, verify"
description: >
  Data plane integration test — end-to-end register, stream, verify

status: work-completed
workflow_type: test
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-08T20:40:46Z
last_update: 2026-03-08T20:42:46Z
date_finished: 2026-03-08T20:42:46Z
---

# T-034: Data plane integration test — end-to-end register, stream, verify

## Context

End-to-end integration tests for the data plane: register session with PTY + data server, connect via binary frames, verify streaming I/O, ping/pong, and capability reporting. Validates T-029/T-030/T-031/T-033 together.

## Acceptance Criteria

### Agent
- [x] Integration test: data plane streams output (inject via control plane, read via data plane)
- [x] Integration test: bidirectional I/O (Input frame in, Output frame out)
- [x] Integration test: ping/pong over data plane
- [x] Integration test: capabilities include data_plane and stream in status
- [x] All 114 tests pass (82 unit + 11 integration + 14 protocol + 7 hub)

## Verification

/Users/dimidev32/.cargo/bin/cargo test --workspace 2>&1 | tail -5

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

### 2026-03-08T20:40:46Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-034-data-plane-integration-test--end-to-end-.md
- **Context:** Initial task creation

### 2026-03-08T20:42:46Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
