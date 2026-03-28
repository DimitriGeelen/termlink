---
id: T-550
name: "Add integration tests for --json flags on ping, clean, tag"
description: >
  Add integration tests for --json flags on ping, clean, tag

status: started-work
workflow_type: test
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T09:48:18Z
last_update: 2026-03-28T09:48:18Z
date_finished: null
---

# T-550: Add integration tests for --json flags on ping, clean, tag

## Context

T-546/T-548/T-549 added --json to ping, clean, tag. Need integration test coverage.

## Acceptance Criteria

### Agent
- [x] Test for `ping --json` validates JSON structure (status, latency_ms, id)
- [x] Test for `clean --json` validates JSON structure (dry_run, count, sessions)
- [x] Test for `tag --json` validates JSON structure after tag update
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

### 2026-03-28T09:48:18Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-550-add-integration-tests-for---json-flags-o.md
- **Context:** Initial task creation
