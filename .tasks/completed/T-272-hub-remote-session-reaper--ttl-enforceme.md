---
id: T-272
name: "Hub remote session reaper — TTL enforcement background task"
description: >
  Hub remote session reaper — TTL enforcement background task

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-25T11:53:40Z
last_update: 2026-03-25T11:55:52Z
date_finished: 2026-03-25T11:53:59Z
---

# T-272: Hub remote session reaper — TTL enforcement background task

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] Hub background task reaps remote sessions that exceed TTL
- [x] Reaper runs periodically without blocking hub operations
- [x] All existing tests pass

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

### 2026-03-25T11:53:40Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-272-hub-remote-session-reaper--ttl-enforceme.md
- **Context:** Initial task creation

### 2026-03-25T11:53:59Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
