---
id: T-896
name: "Standardize MCP resize and request tool outputs to structured JSON"
description: >
  Standardize MCP resize and request tool outputs to structured JSON

status: started-work
workflow_type: refactor
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-05T08:23:28Z
last_update: 2026-04-05T08:23:28Z
date_finished: null
---

# T-896: Standardize MCP resize and request tool outputs to structured JSON

## Context

Final batch of MCP tools that still return plain text format strings instead of structured JSON.

## Acceptance Criteria

### Agent
- [x] termlink_resize returns JSON with ok, cols, rows
- [x] termlink_request returns JSON with ok, request_id, reply_topic, response fields
- [x] termlink_wait returns JSON with ok, topic, event fields (success) or ok:false, error (timeout)
- [x] Integration tests updated (wait_timeout, request_timeout, wait_receives_event)
- [x] All tests pass, zero clippy warnings

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

### 2026-04-05T08:23:28Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-896-standardize-mcp-resize-and-request-tool-.md
- **Context:** Initial task creation
