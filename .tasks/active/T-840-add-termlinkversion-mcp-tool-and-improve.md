---
id: T-840
name: "Add termlink_version MCP tool and improve version command JSON output"
description: >
  Add termlink_version MCP tool and improve version command JSON output

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-04T00:10:51Z
last_update: 2026-04-04T00:10:51Z
date_finished: null
---

# T-840: Add termlink_version MCP tool and improve version command JSON output

## Context

MCP clients need to query TermLink version and available tool count. Add `termlink_version` MCP tool. Also add `termlink_token_create` MCP tool for programmatic token generation (reads session secret, creates scoped token).

## Acceptance Criteria

### Agent
- [x] `termlink_version` MCP tool exists in tools.rs, returns version/commit/target/tool_count JSON
- [x] `termlink_token_create` MCP tool exists with TokenCreateParams, creates scoped capability tokens
- [x] Both tools compile and have unit tests for their param structs (3 new tests)
- [x] All existing tests still pass (752 total)
- [x] Zero clippy warnings

## Verification

cargo test --workspace --lib 2>&1 | tail -5
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

### 2026-04-04T00:10:51Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-840-add-termlinkversion-mcp-tool-and-improve.md
- **Context:** Initial task creation
