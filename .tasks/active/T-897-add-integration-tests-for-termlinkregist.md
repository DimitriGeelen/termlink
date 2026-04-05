---
id: T-897
name: "Add integration tests for termlink_register and termlink_deregister MCP tools"
description: >
  Add integration tests for termlink_register and termlink_deregister MCP tools

status: started-work
workflow_type: test
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-05T08:31:49Z
last_update: 2026-04-05T08:31:49Z
date_finished: null
---

# T-897: Add integration tests for termlink_register and termlink_deregister MCP tools

## Context

T-834 added termlink_register and termlink_deregister MCP tools but only has unit tests for params deserialization. Need integration tests verifying actual endpoint lifecycle.

## Acceptance Criteria

### Agent
- [x] Integration test: register creates an endpoint that appears in list_sessions
- [x] Integration test: deregister removes the endpoint
- [x] Integration test: deregister with invalid ID returns error
- [x] Integration test: register with no parameters (minimal)
- [x] All tests pass (881 total)

## Verification

cargo test --workspace
cargo clippy --workspace --all-targets

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

### 2026-04-05T08:31:49Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-897-add-integration-tests-for-termlinkregist.md
- **Context:** Initial task creation
