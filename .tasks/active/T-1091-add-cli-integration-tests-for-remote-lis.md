---
id: T-1091
name: "Add CLI integration tests for remote list, exec, doctor, and push error paths"
description: >
  Add CLI integration tests for remote list, exec, doctor, and push error paths

status: started-work
workflow_type: test
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-16T21:18:00Z
last_update: 2026-04-16T21:18:00Z
date_finished: null
---

# T-1091: Add CLI integration tests for remote list, exec, doctor, and push error paths

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] Tests for `remote list` error paths (no secret, unreachable hub)
- [x] Tests for `remote exec` error paths (missing command, unreachable hub)
- [x] Tests for `remote doctor` error paths (no secret, unreachable JSON)
- [x] Tests for `remote push` error paths (no file/message, nonexistent file)
- [x] All 8 new tests pass, zero clippy warnings on the test crate

### Human
- [ ] [RUBBER-STAMP] Verify test count increased
  **Steps:** `cd /opt/termlink && cargo test -p termlink -- remote 2>&1 | grep "passed"`
  **Expected:** Additional tests passing beyond baseline
  **If not:** Check test names
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.

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

### 2026-04-16T21:18:00Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1091-add-cli-integration-tests-for-remote-lis.md
- **Context:** Initial task creation
