---
id: T-688
name: "Add cap filter and metadata field to MCP discover and SessionInfo"
description: >
  Add cap filter and metadata field to MCP discover and SessionInfo

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T23:34:50Z
last_update: 2026-03-28T23:34:50Z
date_finished: 2026-03-29T00:16:00Z
---

# T-688: Add cap filter and metadata field to MCP discover and SessionInfo

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] `cap` field added to MCP `DiscoverParams` for capability filtering
- [x] `metadata` field added to MCP `SessionInfo` struct (as serde_json::Value)
- [x] Discover tool filters by capabilities when `cap` provided
- [x] list_sessions mapping includes metadata
- [x] Project compiles cleanly

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

### 2026-03-28T23:34:50Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-688-add-cap-filter-and-metadata-field-to-mcp.md
- **Context:** Initial task creation
