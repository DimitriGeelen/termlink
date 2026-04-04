---
id: T-845
name: "Add termlink_token_inspect MCP tool — decode and inspect capability tokens"
description: >
  Add termlink_token_inspect MCP tool — decode and inspect capability tokens

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-04T09:20:10Z
last_update: 2026-04-04T09:26:32Z
date_finished: 2026-04-04T09:26:32Z
---

# T-845: Add termlink_token_inspect MCP tool — decode and inspect capability tokens

## Context

Add `termlink_token_inspect` MCP tool — decodes a capability token string and returns its payload (session, scope, expiry, expired status). Mirrors `termlink token inspect` CLI command.

## Acceptance Criteria

### Agent
- [x] `termlink_token_inspect` MCP tool added with `token: String` parameter
- [x] Returns JSON with payload fields (session, scope, expires_at) and expired status
- [x] Returns error string for invalid token format (no dot separator)
- [x] Returns error string for invalid base64 payload
- [x] Returns error string for invalid JSON in payload
- [x] Unit test for `TokenInspectParams` deserialization (2 tests)
- [x] Integration test for valid token inspection
- [x] Integration test for invalid token format
- [x] `termlink version` tool count incremented to 39
- [x] `cargo test --workspace` passes (791 tests)
- [x] `cargo clippy --workspace --all-targets` has no warnings

## Verification

cargo test --workspace 2>&1 | tail -3
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

### 2026-04-04T09:20:10Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-845-add-termlinktokeninspect-mcp-tool--decod.md
- **Context:** Initial task creation

### 2026-04-04T09:26:32Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
