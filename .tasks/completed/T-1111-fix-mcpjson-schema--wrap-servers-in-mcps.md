---
id: T-1111
name: "Fix .mcp.json schema — wrap servers in mcpServers key"
description: >
  Fix .mcp.json schema — wrap servers in mcpServers key

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-17T19:45:43Z
last_update: 2026-04-17T20:08:19Z
date_finished: 2026-04-17T20:08:19Z
---

# T-1111: Fix .mcp.json schema — wrap servers in mcpServers key

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `.mcp.json` parses as valid JSON and contains top-level `mcpServers` key wrapping all three servers — verified 2026-04-19 (post-hoc): `python3 -c "import json; d=json.load(open('.mcp.json')); print(list(d['mcpServers'].keys()))"` → `['context7', 'playwright', 'termlink']`
- [x] `claude mcp list` shows context7, playwright, and termlink without schema errors — verified live: MCP tools (context7, playwright, termlink) resolve in this session

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

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.

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

### 2026-04-17T19:45:43Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1111-fix-mcpjson-schema--wrap-servers-in-mcps.md
- **Context:** Initial task creation

### 2026-04-17T20:08:19Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
