---
id: T-556
name: "Add integration tests for exec --json and version --json"
description: >
  Add integration tests for exec --json and version --json

status: work-completed
workflow_type: test
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/tests/cli_integration.rs]
related_tasks: []
created: 2026-03-28T10:03:43Z
last_update: 2026-03-28T10:04:43Z
date_finished: 2026-03-28T10:04:43Z
---

# T-556: Add integration tests for exec --json and version --json

## Context

Cover T-552 (exec --json) and T-540 (version --json) with integration tests.

## Acceptance Criteria

### Agent
- [x] Test for `exec --json` validates JSON with stdout, exit_code
- [x] Test for `version --json` validates JSON with version, commit, target
- [x] Test for `hub status --json` validates JSON structure
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

### 2026-03-28T10:03:43Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-556-add-integration-tests-for-exec---json-an.md
- **Context:** Initial task creation

### 2026-03-28T10:04:43Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
