---
id: T-1096
name: "Add CLI tests for inbox JSON output and token command"
description: >
  Add CLI tests for inbox JSON output and token command

status: work-completed
workflow_type: test
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-16T21:55:34Z
last_update: 2026-04-16T21:58:34Z
date_finished: 2026-04-16T21:58:34Z
---

# T-1096: Add CLI tests for inbox JSON output and token command

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] Test: inbox status --json reports hub-not-running error cleanly
- [x] Test: inbox clear --all --json reports hub-not-running error cleanly
- [x] Test: token create with nonexistent session reports session-not-found
- [x] All 3 new tests pass

### Human
- [ ] [RUBBER-STAMP] Verify test count increased
  **Steps:** `cd /opt/termlink && cargo test -p termlink -- inbox_status_json inbox_clear_json token_create_custom 2>&1 | grep "passed"`
  **Expected:** New tests passing
  **If not:** Check test names


**Agent evidence (auto-batch 2026-04-19, G-008 remediation, test-count, t-1096):** Implementation commit `b1499822` added 3 new test function(s) covering inbox JSON output + token command in `crates/termlink-cli/tests/cli_integration.rs`. Current file holds ~168 tests (grep'd test-attribute or fn-test count). Pre-series baseline was lower; test count clearly increased. RUBBER-STAMPable.

## Verification

bash -c 'cargo test -p termlink -- inbox_status_json inbox_clear_json token_create_no 2>&1 | grep -q "3 passed"'

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

### 2026-04-16T21:55:34Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1096-add-cli-tests-for-inbox-json-output-and-.md
- **Context:** Initial task creation

### 2026-04-16T21:58:34Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed

### 2026-04-16T22:07:03Z — programmatic-evidence [T-1097]
- **Evidence:** 3 inbox JSON + token tests passing: cargo test -p termlink -- inbox_status_json inbox_clear_json token_create_no (3 passed)
- **Verified by:** automated command execution
