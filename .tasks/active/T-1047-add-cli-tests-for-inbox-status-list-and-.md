---
id: T-1047
name: "Add CLI tests for inbox status, list, and clear error paths"
description: >
  Add CLI tests for inbox status, list, and clear error paths

status: started-work
workflow_type: test
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-14T06:37:16Z
last_update: 2026-04-14T06:37:16Z
date_finished: null
---

# T-1047: Add CLI tests for inbox status, list, and clear error paths

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] Test: `termlink inbox status` fails cleanly when hub is not running
- [x] Test: `termlink inbox list <target>` fails cleanly when hub is not running
- [x] Test: `termlink inbox clear --all` fails cleanly when hub is not running
- [x] Test: `termlink inbox clear` (no args) fails with arg validation error
- [x] All 4 tests pass, zero clippy warnings

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

# Shell commands that MUST pass before work-completed. One per line.
cargo test -p termlink --test cli_integration cli_inbox_status_no_hub 2>&1 | grep -q "1 passed"
cargo test -p termlink --test cli_integration cli_inbox_list_no_hub 2>&1 | grep -q "1 passed"
cargo test -p termlink --test cli_integration cli_inbox_clear_all_no_hub 2>&1 | grep -q "1 passed"
cargo test -p termlink --test cli_integration cli_inbox_clear_missing_args 2>&1 | grep -q "1 passed"

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

### 2026-04-14T06:37:16Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1047-add-cli-tests-for-inbox-status-list-and-.md
- **Context:** Initial task creation
