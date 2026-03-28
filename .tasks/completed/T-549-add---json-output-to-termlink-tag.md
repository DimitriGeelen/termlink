---
id: T-549
name: "Add --json output to termlink tag"
description: >
  Add --json output to termlink tag

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/metadata.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-03-28T09:46:16Z
last_update: 2026-03-28T09:47:53Z
date_finished: 2026-03-28T09:47:53Z
---

# T-549: Add --json output to termlink tag

## Context

`termlink tag` outputs text but no `--json` option for scripting.

## Acceptance Criteria

### Agent
- [x] `--json` flag added to Tag command in cli.rs
- [x] `cmd_tag` outputs JSON when --json is set (full session.update result)
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

### 2026-03-28T09:46:16Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-549-add---json-output-to-termlink-tag.md
- **Context:** Initial task creation

### 2026-03-28T09:47:53Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
