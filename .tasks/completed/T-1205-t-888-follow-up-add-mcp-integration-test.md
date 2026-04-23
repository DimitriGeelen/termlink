---
id: T-1205
name: "T-888 follow-up: add MCP integration test for termlink_kv_watch"
description: >
  T-888 follow-up: add MCP integration test for termlink_kv_watch

status: work-completed
workflow_type: test
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-23T16:19:16Z
last_update: 2026-04-23T16:20:47Z
date_finished: 2026-04-23T16:20:47Z
---

# T-1205: T-888 follow-up: add MCP integration test for termlink_kv_watch

## Context

T-888 added the `termlink_kv_watch` MCP tool but did not extend the MCP
integration test suite. The tool-name registration check at
`crates/termlink-mcp/tests/mcp_integration.rs:67` does not include
`termlink_kv_watch`, so a future regression that drops or renames the tool
would not be caught. Add it to the registration list and add a small
end-to-end test that exercises the watch path through the MCP transport.

## Acceptance Criteria

### Agent
- [x] `termlink_kv_watch` added to the expected-tool-names list in `test_list_tools`
- [x] New `tokio::test` `test_kv_watch_observes_change` calls `termlink_kv_set` then `termlink_kv_watch` (with `since=0`) and asserts the returned event payload carries `op=set`, the right key, and the right value
- [x] `cargo test -p termlink-mcp --test mcp_integration` passes
- [x] `cargo build --workspace --quiet` passes

## Verification

cargo test -p termlink-mcp --test mcp_integration test_kv_watch
cargo test -p termlink-mcp --test mcp_integration test_list_tools
grep -q "termlink_kv_watch" crates/termlink-mcp/tests/mcp_integration.rs

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

### 2026-04-23T16:19:16Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1205-t-888-follow-up-add-mcp-integration-test.md
- **Context:** Initial task creation

### 2026-04-23T16:20:47Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
