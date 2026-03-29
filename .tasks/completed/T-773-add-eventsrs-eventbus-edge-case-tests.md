---
id: T-773
name: "Add events.rs EventBus edge case tests"
description: >
  Add events.rs EventBus edge case tests

status: work-completed
workflow_type: test
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-29T23:31:49Z
last_update: 2026-03-29T23:33:06Z
date_finished: 2026-03-29T23:33:06Z
---

# T-773: Add events.rs EventBus edge case tests

## Context

events.rs has 9 tests but is missing: all()/all_by_topic() methods, Event serde roundtrip, poll_topic with no matches, capacity-1 boundary, default capacity.

## Acceptance Criteria

### Agent
- [x] Add test for all() and all_by_topic() methods
- [x] Add test for Event struct serde roundtrip
- [x] Add test for poll_topic with no matching events
- [x] Add test for capacity-1 ring buffer behavior
- [x] Add test for default() constructor uses DEFAULT_CAPACITY
- [x] Add test for large burst overflow with sequence continuity
- [x] All tests pass: `cargo test -p termlink-session -- events` (16 passing)

### Human
<!-- Remove this section — all criteria are agent-verifiable.
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification

cargo test -p termlink-session -- events --quiet

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

### 2026-03-29T23:31:49Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-773-add-eventsrs-eventbus-edge-case-tests.md
- **Context:** Initial task creation

### 2026-03-29T23:33:06Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
