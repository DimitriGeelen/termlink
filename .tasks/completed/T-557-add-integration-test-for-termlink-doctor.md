---
id: T-557
name: "Add integration test for termlink doctor and doctor --json"
description: >
  Add integration test for termlink doctor and doctor --json

status: work-completed
workflow_type: test
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/tests/cli_integration.rs]
related_tasks: []
created: 2026-03-28T10:04:57Z
last_update: 2026-03-28T10:05:48Z
date_finished: 2026-03-28T10:05:48Z
---

# T-557: Add integration test for termlink doctor and doctor --json

## Context

No test coverage for `termlink doctor` or `termlink doctor --json`.

## Acceptance Criteria

### Agent
- [x] Test for `doctor` text output (checks pass, version shown)
- [x] Test for `doctor --json` validates JSON structure (checks array, summary)
- [x] All tests pass

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

### 2026-03-28T10:04:57Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-557-add-integration-test-for-termlink-doctor.md
- **Context:** Initial task creation

### 2026-03-28T10:05:48Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
