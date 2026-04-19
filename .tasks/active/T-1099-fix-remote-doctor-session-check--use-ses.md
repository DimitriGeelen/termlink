---
id: T-1099
name: "Fix remote doctor session check — use session.discover not session.list"
description: >
  Fix remote doctor session check — use session.discover not session.list

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-16T23:37:43Z
last_update: 2026-04-19T16:10:25Z
date_finished: 2026-04-16T23:41:52Z
---

# T-1099: Fix remote doctor session check — use session.discover not session.list

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] Remote doctor uses session.discover instead of session.list for session count
- [x] MCP doctor tool uses same fix (tools.rs line 4716)
- [x] `termlink remote doctor local-test` shows sessions PASS with 7 sessions listed
- [x] All 1,121 tests pass

### Human
- [ ] [RUBBER-STAMP] Run `termlink remote doctor local-test` and verify sessions shows PASS
  **Steps:** `cd /opt/termlink && termlink remote doctor local-test`
  **Expected:** sessions check shows [PASS] with count
  **If not:** Check router method name

  **Agent evidence (2026-04-19):** Ran against current local hub with rebuilt debug CLI. Output:
  ```
  Remote doctor: 127.0.0.1:9100
    [PASS] connectivity: connected in 62ms
    [PASS] sessions: 3 session(s): framework-agent, termlink-agent, ntb-dev-test
    [PASS] inbox: no pending transfers
    Summary: 3 pass, 0 warn, 0 fail
  ```
  session.discover path works and lists named sessions. Human may rubber-stamp.

## Verification

bash -c '! grep -q "call(\"session.list\"" crates/termlink-cli/src/commands/remote.rs'
bash -c '! grep -q "call(\"session.list\"" crates/termlink-mcp/src/tools.rs'

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

### 2026-04-16T23:37:43Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1099-fix-remote-doctor-session-check--use-ses.md
- **Context:** Initial task creation

### 2026-04-16T23:41:52Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
