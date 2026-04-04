---
id: T-842
name: "Add MCP tool count to doctor output and version string"
description: >
  Add MCP tool count to doctor output and version string

status: started-work
workflow_type: build
owner: agent
horizon: later
tags: []
components: []
related_tasks: []
created: 2026-04-04T00:23:35Z
last_update: 2026-04-04T00:23:46Z
date_finished: null
---

# T-842: Add MCP tool count to doctor output and version string

## Context

Add MCP tool count to `termlink version` and `termlink doctor` output so operators can verify MCP capability at a glance. The MCP `termlink_version` tool already returns tool count — this brings parity to the CLI.

## Acceptance Criteria

### Agent
- [x] `termlink_mcp` crate exposes a `pub fn tool_count() -> usize` function
- [x] `termlink version` text output includes MCP tool count (e.g. `termlink 0.9.414 (e6d55ea) [x86_64-unknown-linux-gnu] — 38 MCP tools`)
- [x] `termlink version --json` output includes `"mcp_tools"` field
- [x] `termlink doctor` version check includes MCP tool count
- [x] `cargo test --workspace` passes (755 tests)
- [x] `cargo clippy --workspace --all-targets` has no warnings

## Verification

cargo build --release -p termlink 2>&1 | tail -1
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

### 2026-04-04T00:23:35Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-842-add-mcp-tool-count-to-doctor-output-and-.md
- **Context:** Initial task creation

### 2026-04-04T00:23:46Z — status-update [task-update-agent]
- **Change:** horizon: now → later
