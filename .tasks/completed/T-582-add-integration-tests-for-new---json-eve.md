---
id: T-582
name: "Add integration tests for new --json event and pty commands"
description: >
  Add integration tests for new --json event and pty commands

status: work-completed
workflow_type: test
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/tests/cli_integration.rs]
related_tasks: []
created: 2026-03-28T15:45:41Z
last_update: 2026-03-28T15:47:17Z
date_finished: 2026-03-28T15:47:17Z
---

# T-582: Add integration tests for new --json event and pty commands

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] Test for event topics --json output
- [x] Test for event wait --json with emit trigger
- [x] Test for pty output --json
- [x] All tests pass (54 integration tests)

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

### 2026-03-28T15:45:41Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-582-add-integration-tests-for-new---json-eve.md
- **Context:** Initial task creation

### 2026-03-28T15:47:17Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
