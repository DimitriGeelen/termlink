---
id: T-1098
name: "Add CLI tests for mirror, interact, and agent listen error paths"
description: >
  Add CLI tests for mirror, interact, and agent listen error paths

status: work-completed
workflow_type: test
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-16T23:31:52Z
last_update: 2026-04-16T23:34:47Z
date_finished: 2026-04-16T23:34:47Z
---

# T-1098: Add CLI tests for mirror, interact, and agent listen error paths

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] Tests for `mirror` error paths (no hub, scrollback option parsing)
- [x] Tests for `interact` error paths (missing command, nonexistent session)
- [x] Tests for `agent listen` + `agent ask` error paths (no hub)
- [x] All 6 new tests pass

### Human
- [ ] [RUBBER-STAMP] Verify test count increased
  **Steps:** `cd /opt/termlink && cargo test -p termlink -- mirror interact agent_listen 2>&1 | grep "passed"`
  **Expected:** Additional tests passing
  **If not:** Check test names

## Verification

bash -c 'cargo test -p termlink -- mirror_no_hub mirror_scrollback interact_missing interact_nonexistent_session_with_cmd agent_listen_no_hub agent_ask_no_hub 2>&1 | grep -q "6 passed"'

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

### 2026-04-16T23:31:52Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1098-add-cli-tests-for-mirror-interact-and-ag.md
- **Context:** Initial task creation

### 2026-04-16T23:34:47Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
