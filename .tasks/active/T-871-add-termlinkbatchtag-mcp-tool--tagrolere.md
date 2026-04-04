---
id: T-871
name: "Add termlink_batch_tag MCP tool — tag/role/rename across filtered sessions"
description: >
  Add termlink_batch_tag MCP tool — tag/role/rename across filtered sessions

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-04T21:54:21Z
last_update: 2026-04-04T21:54:21Z
date_finished: null
---

# T-871: Add termlink_batch_tag MCP tool — tag/role/rename across filtered sessions

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] `BatchTagParams` struct with filter (tag/role/name), add_tags, remove_tags, add_roles, remove_roles
- [x] `termlink_batch_tag` tool applies tag/role changes to all matching sessions concurrently
- [x] Returns `{ok, results: [{session, display_name, tags, roles}...], total, succeeded, failed}`
- [x] Unit tests for BatchTagParams (full + minimal)
- [x] MCP integration tests: batch tag no matches + no operation error
- [x] 45 MCP tools total
- [x] Zero clippy warnings

### Human
<!-- No human ACs.
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

### 2026-04-04T21:54:21Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-871-add-termlinkbatchtag-mcp-tool--tagrolere.md
- **Context:** Initial task creation
