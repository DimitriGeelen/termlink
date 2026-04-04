---
id: T-844
name: "Add MCP integration tests for untested tools — signal, output, broadcast, emit_to, inject"
description: >
  Add MCP integration tests for untested tools — signal, output, broadcast, emit_to, inject

status: started-work
workflow_type: test
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-04T09:09:42Z
last_update: 2026-04-04T09:09:42Z
date_finished: null
---

# T-844: Add MCP integration tests for untested tools — signal, output, broadcast, emit_to, inject

## Context

7 of 38 MCP tools have no integration tests: broadcast, emit_to, exec, inject, output, signal, spawn. Add error-path and basic happy-path tests for the feasible ones (signal, output, broadcast, emit_to, inject).

## Acceptance Criteria

### Agent
- [x] Integration test for `termlink_signal` with nonexistent session
- [x] Integration test for `termlink_output` with nonexistent session
- [x] Integration test for `termlink_broadcast` when hub not running
- [x] Integration test for `termlink_emit_to` when hub not running
- [x] Integration test for `termlink_inject` with nonexistent session
- [x] Integration test for `termlink_output` with non-PTY session
- [x] Integration test for `termlink_inject` with registered session
- [x] `cargo test --workspace` passes (787 tests)
- [x] `cargo clippy --workspace --all-targets` has no warnings

## Verification

cargo test -p termlink-mcp --test mcp_integration 2>&1 | tail -5
test "$(cargo clippy --workspace --all-targets 2>&1 | grep -c 'warning:')" = "0"

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

### 2026-04-04T09:09:42Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-844-add-mcp-integration-tests-for-untested-t.md
- **Context:** Initial task creation
