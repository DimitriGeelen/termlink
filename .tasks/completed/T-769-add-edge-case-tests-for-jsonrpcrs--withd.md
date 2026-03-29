---
id: T-769
name: "Add edge case tests for jsonrpc.rs — with_data, deserialization edge cases, RpcResponse dispatch"
description: >
  Add edge case tests for jsonrpc.rs — with_data, deserialization edge cases, RpcResponse dispatch

status: work-completed
workflow_type: test
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-29T23:08:41Z
last_update: 2026-03-29T23:10:06Z
date_finished: 2026-03-29T23:10:06Z
---

# T-769: Add edge case tests for jsonrpc.rs — with_data, deserialization edge cases, RpcResponse dispatch

## Context

`jsonrpc.rs` has 6 tests but misses: `with_data()` constructor, `internal_error()`, request deserialization from raw JSON, `RpcResponse` untagged deserialization from both success/error JSON.

## Acceptance Criteria

### Agent
- [x] Add test for `ErrorResponse::with_data()` including data payload
- [x] Add test for `ErrorResponse::internal_error()` constructor
- [x] Add test for `Request` deserialization from raw JSON string (+ missing params edge case)
- [x] Add test for `RpcResponse` deserialization from success JSON
- [x] Add test for `RpcResponse` deserialization from error JSON
- [x] All new tests pass via `cargo test -p termlink-protocol jsonrpc`

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification

cargo test -p termlink-protocol jsonrpc

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

### 2026-03-29T23:08:41Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-769-add-edge-case-tests-for-jsonrpcrs--withd.md
- **Context:** Initial task creation

### 2026-03-29T23:10:06Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
