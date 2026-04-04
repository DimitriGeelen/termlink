---
id: T-853
name: "Add termlink_overview MCP tool — single-call workspace status summary"
description: >
  Add termlink_overview MCP tool — single-call workspace status summary

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-04T15:35:57Z
last_update: 2026-04-04T15:35:57Z
date_finished: null
---

# T-853: Add termlink_overview MCP tool — single-call workspace status summary

## Context

A single MCP tool call that gives an AI agent the full TermLink workspace status: session count, hub status, runtime directory, version, and summary of sessions. Reduces the need for agents to call list_sessions + hub_status + info separately.

## Acceptance Criteria

### Agent
- [x] MCP tool `termlink_overview` added (no params required)
- [x] Returns JSON with: ok, session_count, sessions (list of {id, name, state, alive, pid, tags, roles}), hub_running, hub_socket, runtime_dir, sessions_dir, version, mcp_tools
- [x] 2 integration tests: empty workspace, workspace with 2 sessions
- [x] tool_count incremented (41 tools)
- [x] All tests pass: cargo test -p termlink-mcp (111 tests)
- [x] Zero clippy warnings: cargo clippy -p termlink-mcp

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

<!-- Shell commands that MUST pass before work-completed. One per line.
     Lines starting with # are comments. Empty lines ignored.
     The completion gate runs each command — if any exits non-zero, completion is blocked.
     Examples:
       python3 -c "import yaml; yaml.safe_load(open('path/to/file.yaml'))"
       curl -sf http://localhost:3000/page
       grep -q "expected_string" output_file.txt
-->

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

### 2026-04-04T15:35:57Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-853-add-termlinkoverview-mcp-tool--single-ca.md
- **Context:** Initial task creation
