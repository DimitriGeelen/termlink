---
id: T-1042
name: "Add CLI tests for version, events, and topics commands"
description: >
  Add CLI integration tests for version output, events on nonexistent session, and topics with no sessions. Fills test coverage gaps.

status: started-work
workflow_type: test
owner: human
horizon: now
tags: []
components: [crates/termlink-cli/tests/cli_integration.rs]
related_tasks: []
created: 2026-04-13T21:41:45Z
last_update: 2026-04-13T21:41:45Z
date_finished: null
---

# T-1042: Add CLI tests for version, events, and topics commands

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] Test: `termlink version` outputs version string
- [x] Test: `termlink version --json` outputs valid JSON with version field
- [x] Test: `termlink events` on nonexistent session returns error (text and JSON)
- [x] Test: `termlink topics --json` with no sessions returns empty
- [x] All 5 new tests pass, zero clippy warnings

### Human
- [ ] [RUBBER-STAMP] Verify test count increased
  **Steps:** `cd /opt/termlink && cargo test -p termlink -- cli_version cli_events cli_topics 2>&1 | grep "passed"`
  **Expected:** 4+ tests passed
  **If not:** Check test filter names

## Verification

cargo test -p termlink -- cli_version 2>&1 | grep "passed"
cargo test -p termlink -- cli_events_nonexistent 2>&1 | grep "passed"
cargo test -p termlink -- cli_topics 2>&1 | grep "passed"
cargo clippy -p termlink -- -D warnings 2>&1 | grep -v "^warning:" | grep -q "Finished"

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

### 2026-04-13T21:41:45Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1042-add-cli-tests-for-version-events-and-top.md
- **Context:** Initial task creation
