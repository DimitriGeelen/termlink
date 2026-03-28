---
id: T-553
name: "Add --json output to termlink hub status"
description: >
  Add --json output to termlink hub status

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T09:54:43Z
last_update: 2026-03-28T09:54:43Z
date_finished: null
---

# T-553: Add --json output to termlink hub status

## Context

`termlink hub status` outputs text but no `--json` for scripting/monitoring.

## Acceptance Criteria

### Agent
- [x] `--json` flag added to HubAction::Status variant in cli.rs
- [x] `cmd_hub_status` outputs JSON with status, pid, socket_path when --json set
- [x] Builds without warnings

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

### 2026-03-28T09:54:43Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-553-add---json-output-to-termlink-hub-status.md
- **Context:** Initial task creation
