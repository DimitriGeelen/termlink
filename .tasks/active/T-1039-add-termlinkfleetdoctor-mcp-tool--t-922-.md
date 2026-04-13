---
id: T-1039
name: "Add termlink_fleet_doctor MCP tool — T-922 codification"
description: >
  Add termlink_fleet_doctor MCP tool — T-922 codification

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-13T19:10:23Z
last_update: 2026-04-13T19:10:23Z
date_finished: null
---

# T-1039: Add termlink_fleet_doctor MCP tool — T-922 codification

## Context

T-922 codification: `termlink fleet doctor` has no MCP exposure. Add `termlink_fleet_doctor` MCP tool that checks all hubs and returns JSON results with diagnostics from T-1034.

## Acceptance Criteria

### Agent
- [x] `termlink_fleet_doctor` MCP tool health-checks all configured hubs
- [x] Returns JSON with per-hub status, latency, diagnostics
- [x] Unit test for params parsing (2 tests)
- [x] Builds with zero clippy warnings

## Verification

cargo build -p termlink 2>&1 | grep -q "Finished"
cargo clippy -p termlink-mcp -- -D warnings 2>&1 | grep -v "^warning:" | grep -q "Finished"
cargo test -p termlink-mcp fleet 2>&1 | grep "passed"

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

### 2026-04-13T19:10:23Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1039-add-termlinkfleetdoctor-mcp-tool--t-922-.md
- **Context:** Initial task creation
