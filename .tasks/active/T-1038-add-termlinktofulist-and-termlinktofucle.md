---
id: T-1038
name: "Add termlink_tofu_list and termlink_tofu_clear MCP tools — T-922 codification"
description: >
  Add termlink_tofu_list and termlink_tofu_clear MCP tools — T-922 codification

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-13T19:05:17Z
last_update: 2026-04-13T19:05:17Z
date_finished: null
---

# T-1038: Add termlink_tofu_list and termlink_tofu_clear MCP tools — T-922 codification

## Context

T-922 codification: every CLI command should be MCP-reachable. T-1035 added `termlink tofu list` and `termlink tofu clear` but no MCP tools. Add them following the existing pattern.

## Acceptance Criteria

### Agent
- [x] `termlink_tofu_list` MCP tool returns JSON list of TOFU entries
- [x] `termlink_tofu_clear` MCP tool removes a specific host:port entry
- [x] MCP unit tests for both tools (3 tests: params parsing, missing required)
- [x] Builds with zero clippy warnings

### Human
- [ ] [RUBBER-STAMP] Verify MCP tool count increased in `termlink doctor`
  **Steps:** `cd /opt/termlink && cargo run -- doctor --json 2>/dev/null | python3 -c "import json,sys; print(json.load(sys.stdin))"`
  **Expected:** Tool count increased by 2
  **If not:** Check MCP tool registration

## Verification

cargo build -p termlink 2>&1 | grep -q "Finished"
cargo clippy -p termlink-mcp -- -D warnings 2>&1 | grep -v "^warning:" | grep -q "Finished"
cargo test -p termlink-mcp tofu 2>&1 | grep "passed"

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

### 2026-04-13T19:05:17Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1038-add-termlinktofulist-and-termlinktofucle.md
- **Context:** Initial task creation
