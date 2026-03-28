---
id: T-547
name: "Add missing help text to event subcommand args"
description: >
  Add missing help text to event subcommand args

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T09:42:15Z
last_update: 2026-03-28T09:42:15Z
date_finished: null
---

# T-547: Add missing help text to event subcommand args

## Context

Investigated: the EventCommand enum (primary subcommands) already has full help text. The missing help is only on hidden backward-compat aliases which don't show in `--help`. No changes needed.

## Acceptance Criteria

### Agent
- [x] All event subcommand positional args have `///` help text (already present in EventCommand enum)
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

### 2026-03-28T09:42:15Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-547-add-missing-help-text-to-event-subcomman.md
- **Context:** Initial task creation
