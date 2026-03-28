---
id: T-691
name: "Add --short flag to hub status for one-line output"
description: >
  Add --short flag to hub status for one-line output

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T23:38:09Z
last_update: 2026-03-28T23:38:09Z
date_finished: 2026-03-29T00:20:00Z
---

# T-691: Add --short flag to hub status for one-line output

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] `--short` flag added to `HubAction::Status` in cli.rs
- [x] `short` param threaded to cmd_hub_status
- [x] Short mode outputs one-line: "running PID" or "not_running" or "stale PID"
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

### 2026-03-28T23:38:09Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-691-add---short-flag-to-hub-status-for-one-l.md
- **Context:** Initial task creation
