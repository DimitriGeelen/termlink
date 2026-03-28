---
id: T-682
name: "Add capabilities and metadata to MCP SessionInfo and discover output"
description: >
  Add capabilities and metadata to MCP SessionInfo and discover output

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T22:59:02Z
last_update: 2026-03-28T23:00:04Z
date_finished: 2026-03-29T00:00:30Z
---

# T-682: Add capabilities and metadata to MCP SessionInfo and discover output

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] SessionInfo struct includes capabilities field
- [x] list_sessions maps capabilities from registration
- [x] discover output includes metadata field
- [x] Project compiles cleanly

### Human
<!-- Remove if not needed.
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

### 2026-03-28T22:59:02Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-682-add-capabilities-and-metadata-to-mcp-ses.md
- **Context:** Initial task creation
